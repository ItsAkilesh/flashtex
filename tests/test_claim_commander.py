import copy
from datetime import datetime,timezone
import importlib.util
from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).parents[1]/'scripts'))
import claim_commander as m
NOW=datetime(2026,9,12,8,tzinfo=timezone.utc)
class GateTests(unittest.TestCase):
    def setUp(self):
        self.config={'mode':'read_only_standby','process':{'pid':1,'start_ticks':2,'boot_id':'boot'},'predecessor_id':'astra','publisher_services':['dispatcher']}
        self.authority={'commander_id':'astra','authority_state':'active','machine':'linux-primary'}
        self.receipt={'state':'terminal_quiescent_observed','claim_authorized':False,'predecessor_id':'astra','process_pin':self.config['process'],'observed_main_sha':'a'*40,'observed_utc':NOW.isoformat(),'reasons':[],'publisher_processes':[],'journal_blockers':[],'services':[{'unit':'dispatcher','pid':0,'active':'inactive'}]}
        self.rows=[dict(machine='daniel',observed_utc=NOW.isoformat(),registered=True,working_verified=True,billing_authorized=True,orchestration_capable=True,usable_capacity_verified=True,session_evidence='live-session',remaining=80,comparison_profile='samewindow')]
    def run_gate(self):
        return m.gate(self.config,self.authority,{'state':'running'},self.receipt,'a'*40,self.rows,'daniel',NOW)
    def test_valid(self): self.assertEqual(self.run_gate()['selected'],'daniel')
    def test_blockers(self):
        for key,value in [('state','blocked'),('claim_authorized',True),('observed_main_sha','b'*40),('observed_utc','2026-09-12T07:00:00Z'),('journal_blockers',['pending']),('publisher_processes',[{'pid':9}]),('services',[])]:
            with self.subTest(key=key):
                old=copy.deepcopy(self.receipt);self.receipt[key]=value
                with self.assertRaises(ValueError): self.run_gate()
                self.receipt=old
    def test_competing_authority(self):
        self.authority['commander_id']='other'
        with self.assertRaises(ValueError):self.run_gate()
    def test_unknown_capacity_and_unregistered(self):
        self.rows[0]['registered']=False
        with self.assertRaises(ValueError):self.run_gate()
    def test_predecessor_cannot_win_from_old_live_record(self):
        old=copy.deepcopy(self.rows[0]);old.update(machine='linux-primary',remaining=100)
        self.rows.append(old);self.assertEqual(self.run_gate()['selected'],'daniel')

class RealGitTests(unittest.TestCase):
    def test_isolated_nonforce_claim_and_duplicate_refusal(self):
        import json
        import subprocess
        import tempfile
        with tempfile.TemporaryDirectory() as temporary:
            base=Path(temporary);remote=base/'remote.git';repo=base/'repo'
            def git(*args,cwd=None):
                return subprocess.check_output(['git',*args],cwd=cwd,text=True,stderr=subprocess.DEVNULL).strip()
            git('init','--bare',str(remote));git('init','-b','main',str(repo))
            git('config','user.name','Test Agent',cwd=repo);git('config','user.email','test@invalid',cwd=repo)
            (repo/'coordination/failover').mkdir(parents=True)
            now=datetime.now(timezone.utc)
            config={'mode':'read_only_standby','process':{'pid':1,'start_ticks':2,'boot_id':'b'},'predecessor_id':'astra','publisher_services':['dispatcher'],'witness_branch':'agent/orchestrator-witness/test'}
            authority={'commander_id':'astra','authority_state':'active','machine':'linux-primary'}
            for name,value in [('failover',config),('authority',authority),('control',{'state':'running'})]:
                (repo/f'coordination/{name}.json').write_text(json.dumps(value))
            git('add','.',cwd=repo);git('commit','-m','base',cwd=repo);sha=git('rev-parse','HEAD',cwd=repo)
            git('remote','add','origin',str(remote),cwd=repo);git('push','origin','main',cwd=repo)
            git('checkout','-b',config['witness_branch'],cwd=repo)
            receipt={'state':'terminal_quiescent_observed','claim_authorized':False,'predecessor_id':'astra','process_pin':config['process'],'observed_main_sha':sha,'observed_utc':now.isoformat(),'reasons':[],'publisher_processes':[],'journal_blockers':[],'services':[{'unit':'dispatcher','pid':0,'active':'inactive'}]}
            (repo/'coordination/failover/witness-linux.json').write_text(json.dumps(receipt));git('add','.',cwd=repo)
            git('-c','user.name=FlashTeX Witness','-c','user.email=witness@flashtex.invalid','commit','-m','terminal','-m','Commit-Executor: commander_failover.py deterministic local witness',cwd=repo)
            witness=git('rev-parse','HEAD',cwd=repo);git('push','origin','HEAD',cwd=repo);git('checkout','main',cwd=repo)
            evidence=base/'evidence.json';evidence.write_text(json.dumps([dict(machine='daniel',observed_utc=now.isoformat(),registered=True,working_verified=True,billing_authorized=True,orchestration_capable=True,usable_capacity_verified=True,session_evidence='test-session',remaining=5,comparison_profile='test')]))
            journal=base/'claim.json'
            result=m.claim(repo,evidence,'daniel','orchestrator-daniel',witness,journal,'deterministic test','Test User <user@invalid>')
            self.assertEqual(result['state'],'published')
            claimed=json.loads(git('show','origin/main:coordination/authority.json',cwd=repo));self.assertEqual(claimed['commander_id'],'orchestrator-daniel')
            self.assertEqual(git('show','-s','--format=%P','origin/main',cwd=repo),sha)
            with self.assertRaises(ValueError):m.claim(repo,evidence,'daniel','orchestrator-daniel',witness,journal,'test','Test <test@invalid>')
            with self.assertRaises(ValueError):m.claim(repo,evidence,'daniel','orchestrator-daniel',witness,base/'second.json','test','Test <test@invalid>')
