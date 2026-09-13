"""Python port of PR #171's measure.sh. usage: measure.py <render-bin> <out-dir> [ref]

Paths are resolved from the environment, so the tool works on any machine:
  FT_WORKTREE  repo root holding fixtures/ and apps/mac/Fonts (default: the repo this file lives in)
  FT_PDFLATEX  pdflatex to use as the oracle (default: pdflatex from PATH)
"""
import os, shutil, subprocess, sys

S = os.path.dirname(os.path.abspath(__file__))
R = os.environ.get('FT_WORKTREE') or subprocess.run(
    ['git', '-C', S, 'rev-parse', '--show-toplevel'],
    capture_output=True, text=True).stdout.strip()
PDFLATEX = os.environ.get('FT_PDFLATEX') or shutil.which('pdflatex')
BIN, O = sys.argv[1], sys.argv[2]
ref = len(sys.argv) > 3 and sys.argv[3] == 'ref'
os.makedirs(O + '/ours', exist_ok=True); os.makedirs(O + '/ref', exist_ok=True)
if ref:
    if not PDFLATEX:
        sys.exit('no pdflatex found; set FT_PDFLATEX')
    for f in ('hw1/HW1.tex', 'hw2/HW2.tex'):
        shutil.copy(f'{R}/fixtures/real-world/{f}', O + '/ref/')
    env = dict(os.environ, SOURCE_DATE_EPOCH='0', FORCE_SOURCE_DATE='1')
    for n in ('HW1', 'HW2'):
        for _ in range(2):
            subprocess.run([PDFLATEX, '-interaction=batchmode', n + '.tex'], cwd=O + '/ref', env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
# Respect an exported font environment (fontenv-nixos.sh); only fall back
# to the bundled outlines when the caller set nothing. Overriding it here
# is how the oracle harnesses used to discard a correct configuration.
env = dict(os.environ)
env.setdefault('FLASHTEX_FONT_DIRS', R + '/apps/mac/Fonts')
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
