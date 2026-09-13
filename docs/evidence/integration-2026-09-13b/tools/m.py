"""Merge helper. usage:
  m.py start <pr> <branch> <oid>   -> git merge --no-ff --no-commit origin/<branch>; lists conflicts
  m.py commit <pr> <branch> <oid> [note]  -> commits the merge with identity + trailers
  m.py plain "<subject>"           -> commits staged changes with identity + trailers
  m.py abort

Paths are resolved from the environment, so the tool works on any machine:
  FT_WORKTREE  worktree to operate on (default: the repo this file lives in)
  FT_SCRATCH   scratch dir for the commit message (default: $TMPDIR/flashtex-integration-b)
  FT_HOST      host label recorded in the Implementation-Agent trailer
"""
import os, subprocess, sys, tempfile

_here = os.path.dirname(os.path.abspath(__file__))
W = os.environ.get('FT_WORKTREE') or subprocess.run(
    ['git', '-C', _here, 'rev-parse', '--show-toplevel'],
    capture_output=True, text=True).stdout.strip()
S = os.environ.get('FT_SCRATCH') or os.path.join(tempfile.gettempdir(), 'flashtex-integration-b')
os.makedirs(S, exist_ok=True)
HOST = os.environ.get('FT_HOST', 'nixos-pc-kabir')
MSG = os.path.join(S, 'msg.txt')
TRAILERS = f"""
Implementation-Agent: Claude Opus 5 (subagent kabir-claude integration, {HOST})
Commit-Executor: Claude Code direct Git
Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Xd5Hmwh5GHNTiAmHUJ1MZu
"""
def g(*a, check=False):
    r = subprocess.run(['git', '-C', W, *a], capture_output=True, text=True)
    if check and r.returncode:
        print(r.stdout, r.stderr); sys.exit(1)
    return r
def commit(msg):
    open(MSG, 'w').write(msg)
    r = g('-c', 'user.name=GoKubar', '-c', 'user.email=kabirgoyal@icloud.com', 'commit', '-q', '-F', MSG)
    print(r.stdout, r.stderr)
    print(g('log', '--oneline', '-1').stdout)
cmd = sys.argv[1]
if cmd == 'start':
    pr, br, oid = sys.argv[2:5]
    tip = g('rev-parse', 'origin/' + br).stdout.strip()
    if not tip.startswith(oid):
        print(f'WARNING origin/{br} is {tip[:8]}, PR head was {oid}')
    r = g('merge', '--no-ff', '--no-commit', 'origin/' + br)
    print(r.stdout[-3000:], r.stderr[-2000:])
    print('UNMERGED:', g('diff', '--name-only', '--diff-filter=U').stdout)
elif cmd == 'commit':
    pr, br, oid = sys.argv[2:5]
    note = sys.argv[5] if len(sys.argv) > 5 else ''
    commit(f"integration: merge #{pr} ({br.split('/')[-1]} @ {oid[:8]})\n\n" + (note + '\n' if note else '') + TRAILERS)
elif cmd == 'plain':  # non-merge commit of staged changes; message from argv[2]
    commit(sys.argv[2] + '\n' + TRAILERS)
elif cmd == 'abort':
    print(g('merge', '--abort').stderr)
