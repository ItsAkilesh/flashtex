"""Conflict hunk tool.
  resolve.py show <file> [ctx]          print every hunk with its index and ctx lines of context
  resolve.py pick <file> <spec>         spec: comma list of idx=ours|theirs|both|theirs-first
                                        (hunks not named are left as is)
"""
import re, sys

def hunks(lines):
    out, i = [], 0
    while i < len(lines):
        if lines[i].startswith('<<<<<<< '):
            s = i
            m = next(j for j in range(i, len(lines)) if lines[j].startswith('======='))
            e = next(j for j in range(m, len(lines)) if lines[j].startswith('>>>>>>> '))
            out.append((s, m, e)); i = e + 1
        else:
            i += 1
    return out

cmd, path = sys.argv[1], sys.argv[2]
lines = open(path).read().split('\n')
hs = hunks(lines)
if cmd == 'show':
    ctx = int(sys.argv[3]) if len(sys.argv) > 3 else 6
    for k, (s, m, e) in enumerate(hs):
        print(f'### hunk {k} lines {s+1}-{e+1}')
        for j in range(max(0, s - ctx), min(len(lines), e + ctx + 1)):
            print(f'{j+1:6} {lines[j]}')
elif cmd == 'pick':
    spec = dict(x.split('=') for x in sys.argv[3].split(','))
    for k in sorted((int(k) for k in spec), reverse=True):
        s, m, e = hs[k]
        ours, theirs = lines[s+1:m], lines[m+1:e]
        how = spec[str(k)]
        new = {'ours': ours, 'theirs': theirs, 'both': ours + theirs, 'theirs-first': theirs + ours}[how]
        lines[s:e+1] = new
    open(path, 'w').write('\n'.join(lines))
    print('remaining hunks:', len(hunks(lines)))
