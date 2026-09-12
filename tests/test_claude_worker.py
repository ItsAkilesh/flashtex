"""Synthetic auth/model/Cursor doubles: never consumes actual provider usage."""
import importlib.util
import json
import hashlib
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

SCRIPTS = Path(__file__).resolve().parents[1] / 'scripts'
with patch.object(sys, 'path', [str(SCRIPTS)] + sys.path):
    spec = importlib.util.spec_from_file_location('claude_worker_under_test', SCRIPTS/'claude_worker.py')
    cw = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(cw)


class ClaudeWorkerTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        self.enterContext(patch.dict(os.environ, {}, clear=True))
        self.args = SimpleNamespace(id='tester', machine='fixture', cycle_seconds=60,
            funding='api', api_grant=None)
        self.assignment = dict(schema_version=1,task_id='FT-TEST',revision=1,agent_id='tester',
            branch='agent/tester/task',state='assigned',timebox_minutes=10,
            allocation_id='synthetic',owned_paths=['src'],acceptance=['fixture'],objective='fixture')
        self.result = dict(state='in_progress',summary='fixture complete',next_action='continue',
            eta_minutes=[1,2,3],tests=['synthetic check'],reviewed_peers=[],adaptation='no changes')

    def eligible_auth(self):
        return SimpleNamespace(stdout=json.dumps({'loggedIn':True,'authMethod':'claude.ai'}),stderr='',returncode=0)

    def setup_git(self):
        c=cw.coord
        c.git(self.root,'init','--initial-branch=main')
        c.git(self.root,'config','user.name','Synthetic Test')
        c.git(self.root,'config','user.email','fixture@example.invalid')
        c.git(self.root,'config','commit.gpgsign','false')
        c.write_json(self.root/'coordination/assignments/FT-TEST.json',self.assignment)
        c.git(self.root,'add','.')
        c.git(self.root,'commit','-m','synthetic assignment')
        c.git(self.root,'update-ref','refs/remotes/origin/main','HEAD')
        c.git(self.root,'switch','-c',self.assignment['branch'])
        self.state={'phase':'idle','cycles':0}
        self.state_path=c.local_state(self.root)/'claude-worker.json'
        self.grant()

    def fake_model(self,command,root,prompt,seconds,log):
        state=json.loads(self.state_path.read_text())
        self.assertEqual(state['phase'],'model_running')
        self.assertGreaterEqual(state['cycles'],1)
        (root/'src').mkdir(exist_ok=True)
        (root/'src/fixture').write_text('bounded implementation\n')
        log.write_text(json.dumps({'type':'result','subtype':'success','is_error':False,
            'structured_output':self.result,'usage':{'input_tokens':123,'output_tokens':45},'total_cost_usd':0.01}))
        return 0

    def fake_publish(self,root,args):
        cw.coord.git(root,'commit','-m','synthetic checkpoint')

    def test_missing_key_blocks_without_subscription_fallback(self):
        with self.assertRaisesRegex(ValueError,'subscription login is prohibited'):
            cw.check_auth(self.root)

    def test_key_presence_does_not_claim_verified_auth_or_credits(self):
        os.environ['ANTHROPIC_API_KEY']='synthetic-key'
        result=cw.check_auth(self.root)
        self.assertFalse(result['authentication_verified'])
        self.assertEqual(result['remaining_credits'],'unverified')

    def test_oauth_override_is_rejected(self):
        os.environ['ANTHROPIC_API_KEY']='synthetic-key'
        os.environ['CLAUDE_CODE_OAUTH_TOKEN']='synthetic-token'
        with self.assertRaisesRegex(ValueError,'refuses'):
            cw.check_auth(self.root)

    def test_subscription_route_is_unconditionally_rejected(self):
        self.args.funding='subscription'
        with self.assertRaisesRegex(ValueError,'API-only'):
            cw.funded_call(self.root,self.args,self.assignment,{})

    def test_headless_command_never_prompts_or_implicitly_uses_api(self):
        args=cw.command_for()
        self.assertIn('opus',args)
        self.assertIn('dontAsk',args)
        self.assertEqual(args[args.index('--permission-prompts')+1],'none')
        self.assertIn('--bare',args)
        self.assertNotIn('--dangerously-skip-permissions',args)

    def test_usage_is_sanitized_and_not_a_claim_of_actual_spend(self):
        usage=cw.usage_summary({'total_cost_usd':2.5,'usage':{'input_tokens':12,'private_key':'secret'},'private':'secret'})
        self.assertEqual(usage['client_estimated_equivalent_usd'],2.5)
        self.assertEqual(usage['remaining_quota'],'unknown')
        self.assertNotIn('secret',json.dumps(usage))

    def test_successful_cycle_reports_usage_and_only_cursor_publishes(self):
        self.setup_git()
        with patch.object(cw.worker,'execute',side_effect=self.fake_model) as execute, patch.object(cw.coord,'publish',side_effect=self.fake_publish) as publish:
            self.assertEqual(cw.cycle(self.root,self.args,self.assignment,self.state_path,self.state),'in_progress')
            execute.assert_called_once();publish.assert_called_once()
        report=json.loads((self.root/'coordination/agents/tester.json').read_text())
        self.assertEqual(report['claude_usage']['tokens']['input_tokens'],123)
        self.assertEqual(report['supervisor']['assignment'],'FT-TEST:1')
        self.assertEqual(self.state['phase'],'idle')

    def test_finished_assignment_does_not_spend_while_polling(self):
        state={'completed_assignment':'FT-TEST:1'}
        with patch.object(cw.worker,'execute') as execute:
            self.assertEqual(cw.cycle(self.root,self.args,self.assignment,self.root/'state',state),'waiting_for_next_assignment')
            execute.assert_not_called()

    def test_ambiguous_timeout_is_persisted_and_never_automatically_repeated(self):
        self.setup_git()
        with patch.object(cw.worker,'execute',side_effect=subprocess.TimeoutExpired('fixture',60)) as execute:
            with self.assertRaises(subprocess.TimeoutExpired):
                cw.cycle(self.root,self.args,self.assignment,self.state_path,self.state)
            self.assertEqual(json.loads(self.state_path.read_text())['phase'],'model_running')
            with self.assertRaisesRegex(ValueError,'reconciliation'):
                cw.cycle(self.root,self.args,self.assignment,self.state_path,self.state)
            execute.assert_called_once()

    def test_outside_owned_paths_never_publishes(self):
        self.setup_git()
        def model(*args):
            result=self.fake_model(*args)
            (self.root/'unassigned').write_text('bad scope')
            return result
        with patch.object(cw.worker,'execute',side_effect=model),patch.object(cw.coord,'publish') as publish:
            with self.assertRaisesRegex(ValueError,'unassigned paths'):
                cw.cycle(self.root,self.args,self.assignment,self.state_path,self.state)
            publish.assert_not_called()

    def test_cursor_failure_does_not_repeat_paid_implementation(self):
        self.setup_git()
        with patch.object(cw.worker,'execute',side_effect=self.fake_model) as execute, patch.object(cw.coord,'publish',side_effect=RuntimeError('synthetic publication failure')):
            with self.assertRaises(RuntimeError):
                cw.cycle(self.root,self.args,self.assignment,self.state_path,self.state)
            self.assertEqual(self.state['phase'],'publication_started')
            with self.assertRaisesRegex(ValueError,'reconciliation'):
                cw.cycle(self.root,self.args,self.assignment,self.state_path,self.state)
            execute.assert_called_once()

    def test_blocked_result_creates_issue_before_waiting_for_dispatch(self):
        self.setup_git();self.result['state']='blocked'
        with patch.object(cw.worker,'execute',side_effect=self.fake_model),patch.object(cw.coord,'publish',side_effect=self.fake_publish),patch.object(cw,'report_blocker') as notify:
            cw.cycle(self.root,self.args,self.assignment,self.state_path,self.state)
            notify.assert_called_once()
            self.assertEqual(self.state['completed_assignment'],'FT-TEST:1')

    def test_grant_cannot_follow_replaced_api_key(self):
        self.grant();os.environ['ANTHROPIC_API_KEY']='different-account'
        with self.assertRaisesRegex(ValueError,'unverified'):
            cw.funded_call(self.root,self.args,self.assignment,{})

    def test_pending_blocker_delivery_survives_restart_without_inference(self):
        state={'completed_assignment':'FT-TEST:1','pending_blocker':True,'phase':'idle'}
        path=self.root/'state.json'
        with patch.object(cw,'report_blocker',side_effect=RuntimeError('offline')):
            with self.assertRaises(RuntimeError):
                cw.cycle(self.root,self.args,self.assignment,path,state)
        self.assertTrue(state['pending_blocker'])
        with patch.object(cw,'report_blocker') as notify,patch.object(cw.worker,'execute') as execute:
            self.assertEqual(cw.cycle(self.root,self.args,self.assignment,path,state),'waiting_for_next_assignment')
            notify.assert_called_once();execute.assert_not_called()
        self.assertNotIn('pending_blocker',json.loads(path.read_text()))

    def test_invalid_reservation_never_increases_credit_allowance(self):
        self.grant()
        for value in [-1,float('nan'),float('inf'),True]:
            with self.subTest(value=value),self.assertRaisesRegex(ValueError,'reservation'):
                cw.funded_call(self.root,self.args,self.assignment,{'credit_reservations':{'synthetic':value}})

    def test_new_assignment_revision_replenishes_finished_worker(self):
        self.setup_git()
        self.result['state']='ready_for_integration'
        with patch.object(cw.worker,'execute',side_effect=self.fake_model) as execute, patch.object(cw.coord,'publish',side_effect=self.fake_publish):
            cw.cycle(self.root,self.args,self.assignment,self.state_path,self.state)
            self.assignment['revision']=2
            cw.coord.write_json(self.root/'coordination/assignments/FT-TEST.json',self.assignment)
            cw.coord.git(self.root,'add','.')
            cw.coord.git(self.root,'commit','-m','synthetic second assignment')
            cw.coord.git(self.root,'update-ref','refs/remotes/origin/main','HEAD')
            cw.cycle(self.root,self.args,self.assignment,self.state_path,self.state)
            self.assertEqual(execute.call_count,2)
            self.assertEqual(self.state['completed_assignment'],'FT-TEST:2')

    def test_result_rejects_negative_eta_and_multiple_results(self):
        bad=dict(self.result,eta_minutes=[-1,0,1])
        with self.assertRaises(ValueError):cw.validate_result(bad)
        log=self.root/'log'
        log.write_text('\n'.join([json.dumps({'type':'result'})]*2))
        with self.assertRaisesRegex(ValueError,'ambiguous'):cw.read_result(log)

    def test_api_reservation_is_persisted_before_process_launch(self):
        self.setup_git();self.grant()
        def model(*args):
            state=json.loads(self.state_path.read_text())
            self.assertEqual(state['credit_reservations']['synthetic'],1)
            return self.fake_model(*args)
        with patch.object(cw.worker,'execute',side_effect=model),patch.object(cw.coord,'publish',side_effect=self.fake_publish):
            cw.cycle(self.root,self.args,self.assignment,self.state_path,self.state)
        self.assertEqual(self.state['credit_reservations']['synthetic'],1)

    def grant(self):
        self.args.funding='api'
        private = self.root/'.git' if (self.root/'.git').exists() else self.root
        self.args.api_grant=str(private/'grant.json')
        os.environ['ANTHROPIC_API_KEY']='synthetic-never-used'
        grant=dict(credential_sha256=hashlib.sha256(b'synthetic-never-used').hexdigest(),verified_api_auth=True,verified_prepaid_credits=True,provider_spend_cap_verified=True,
            agent_id='tester',allocation_id='synthetic',worktree=str(self.root.resolve()),
            expires_utc='2099-01-01T00:00:00Z',max_total_usd=2,max_call_usd=1)
        Path(self.args.api_grant).write_text(json.dumps(grant))
        return grant

    def test_verified_api_grant_reserves_before_call_and_exhausts(self):
        self.grant();state={}
        auth,cmd=cw.funded_call(self.root,self.args,self.assignment,state)
        self.assertEqual(auth['call_cap_usd'],1)
        self.assertIn('--bare',cmd)
        self.assertIn('--max-budget-usd',cmd)
        cw.funded_call(self.root,self.args,self.assignment,state)
        with self.assertRaisesRegex(ValueError,'exhausted'):
            cw.funded_call(self.root,self.args,self.assignment,state)

    def test_unverified_expired_and_wrong_owner_grants_are_rejected(self):
        for field,value in [('verified_prepaid_credits',False),('agent_id','other'),('expires_utc','2000-01-01T00:00:00Z')]:
            with self.subTest(field=field):
                grant=self.grant();grant[field]=value
                Path(self.args.api_grant).write_text(json.dumps(grant))
                with self.assertRaises(ValueError):
                    cw.funded_call(self.root,self.args,self.assignment,{})

    def test_distinct_blockers_are_reported_same_blocker_deduplicated(self):
        with patch.object(cw.coord,'local_state',return_value=self.root),patch.object(cw.coord,'run',return_value=SimpleNamespace(stdout='https://example.invalid/issue')) as run:
            cw.report_blocker(self.root,self.args,ValueError('auth'))
            cw.report_blocker(self.root,self.args,ValueError('auth'))
            self.assertEqual(run.call_count,1)
            cw.report_blocker(self.root,self.args,RuntimeError('quota'))
            self.assertEqual(run.call_count,2)

    def test_failed_result_not_relabelled_success(self):
        log=self.root/'result'
        log.write_text(json.dumps({'subtype':'error_during_execution','is_error':True,'structured_output':self.result}))
        with self.assertRaises(ValueError):cw.read_result(log)

    def test_stop_control_does_not_start_model(self):
        self.args.once=True
        with patch.object(sys,'argv',['claude_worker.py']),patch.object(cw.coord,'root_dir',return_value=self.root),patch.object(cw.coord,'local_state',return_value=self.root),patch.object(cw,'check_auth',return_value={}),patch.object(cw.coord,'run',return_value=SimpleNamespace(stdout='Logged in',stderr='')),patch.object(cw.coord,'current_git_user_trailer'),patch.object(cw.coord,'checkpoint',return_value={}),patch.object(cw.coord,'peer_json',return_value={'state':'user_stopped'}),patch.object(cw.worker,'execute') as execute:
            self.assertEqual(cw.main(),0)
            execute.assert_not_called()


if __name__=='__main__':unittest.main()
