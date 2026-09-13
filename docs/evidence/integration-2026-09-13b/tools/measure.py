"""Python port of PR #171's measure.sh. usage: measure.py <render-bin> <out-dir> [ref]"""
import os, shutil, subprocess, sys
S = os.path.dirname(os.path.abspath(__file__))
R = '/Users/kubar/code/flashtex/.claude/worktrees/agent-ae6a49305cc843e69'
BIN, O = sys.argv[1], sys.argv[2]
ref = len(sys.argv) > 3 and sys.argv[3] == 'ref'
os.makedirs(O + '/ours', exist_ok=True); os.makedirs(O + '/ref', exist_ok=True)
if ref:
    for f in ('hw1/HW1.tex', 'hw2/HW2.tex'):
        shutil.copy(f'{R}/fixtures/real-world/{f}', O + '/ref/')
    env = dict(os.environ, SOURCE_DATE_EPOCH='0', FORCE_SOURCE_DATE='1')
    for n in ('HW1', 'HW2'):
        for _ in range(2):
            subprocess.run(['/Library/TeX/texbin/pdflatex', '-interaction=batchmode', n + '.tex'], cwd=O + '/ref', env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
env = dict(os.environ, FLASHTEX_FONT_DIRS=R + '/apps/mac/Fonts')
for h in ('hw1/HW1', 'hw2/HW2'):
    n = h.split('/')[1]
    with open(f'{O}/ours/{n}.out', 'w') as out:
        rc = subprocess.run([BIN, '--tex', h + '.tex', '--v2', f'{O}/ours/{n}.json', '--pdf', f'{O}/ours/{n}.pdf'], cwd=R + '/fixtures/real-world', env=env, stdout=out, stderr=subprocess.STDOUT).returncode
    txt = open(f'{O}/ours/{n}.out').read()
    print(n, 'exit', rc, 'overfull-lines', sum('overfull' in l.lower() for l in txt.splitlines()), 'error-lines', sum('error' in l.lower() for l in txt.splitlines()))
    print('  tail:', txt.strip().splitlines()[-1][:300] if txt.strip() else '')
with open(O + '/residuals.md', 'w') as f:
    subprocess.run(['python3', S + '/residuals.py', R, O], stdout=f)
print(open(O + '/residuals.md').read()[:2500])
