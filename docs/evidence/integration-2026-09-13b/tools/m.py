"""Merge helper. usage:
  m.py start <pr> <branch> <oid>   -> git merge --no-ff --no-commit origin/<branch>; lists conflicts
  m.py commit <pr> <branch> <oid> [note]  -> commits the merge with identity + trailers
  m.py abort
"""
import subprocess, sys
W = '/Users/kubar/code/flashtex/.claude/worktrees/agent-ae6a49305cc843e69'
S = '/private/tmp/claude-501/-Users-kubar-code-flashtex/725966af-c852-4dd0-a718-6ce11942557d/scratchpad/integration-b/'
TRAILERS = """
Implementation-Agent: Claude Opus 5 (subagent kabir-claude integration, mac-m5pro-kabir)
Commit-Executor: Claude Code direct Git
Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Xd5Hmwh5GHNTiAmHUJ1MZu
"""
def g(*a, check=False):
    r = subprocess.run(['git', '-C', W, *a], capture_output=True, text=True)
    if check and r.returncode:
        print(r.stdout, r.stderr); sys.exit(1)
    return r
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
    msg = f"integration: merge #{pr} ({br.split('/')[-1]} @ {oid[:8]})\n\n" + (note + '\n' if note else '') + TRAILERS
    open(S + 'msg.txt', 'w').write(msg)
    r = g('-c', 'user.name=GoKubar', '-c', 'user.email=kabirgoyal@icloud.com', 'commit', '-q', '-F', S + 'msg.txt')
    print(r.stdout, r.stderr)
    print(g('log', '--oneline', '-1').stdout)
elif cmd == 'plain':  # non-merge commit of staged changes; message from argv[2]
    open(S + 'msg.txt', 'w').write(sys.argv[2] + '\n' + TRAILERS)
    r = g('-c', 'user.name=GoKubar', '-c', 'user.email=kabirgoyal@icloud.com', 'commit', '-q', '-F', S + 'msg.txt')
    print(r.stdout, r.stderr)
    print(g('log', '--oneline', '-1').stdout)
elif cmd == 'abort':
    print(g('merge', '--abort').stderr)
