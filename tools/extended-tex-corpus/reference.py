#!/usr/bin/env python3
"""Development-only reference engines. Never imported by the FlashTeX product."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
CORPUS = ROOT / 'tests/extended-tex-corpus'

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def executable(name, texbin):
    local = texbin / name
    return str(local) if local.is_file() else shutil.which(name)

def run(argv, cwd, env, timeout):
    start = time.monotonic()
    try:
        p = subprocess.run(argv, cwd=cwd, env=env, stdout=subprocess.PIPE,
                           stderr=subprocess.STDOUT, timeout=timeout)
        return {'argv': argv, 'returncode': p.returncode,
                'seconds': round(time.monotonic()-start, 3),
                'output': p.stdout.decode('utf-8', errors='replace')}
    except subprocess.TimeoutExpired as e:
        return {'argv': argv, 'returncode': None, 'timeout': True,
                'seconds': round(time.monotonic()-start, 3),
                'output': (e.stdout or b'').decode('utf-8', errors='replace')}

def validate(manifest):
    ids = set()
    for case in manifest['cases']:
        assert case['id'] not in ids, 'duplicate case'
        ids.add(case['id'])
        base = CORPUS / 'cases' / case['id']
        assert case['entry'] in case['files']
        assert case['engine'] in ('pdftex', 'pdflatex', 'lualatex', 'xelatex')
        assert case['expect'] in ('success', 'error')
        for name in case['files']:
            p = base / name
            assert p.is_file() and p.resolve().is_relative_to(base.resolve())
        assert sorted(case['files']) == sorted(str(p.relative_to(base)) for p in base.rglob('*') if p.is_file())

def build(case, out, args, env):
    work = out / case['id']
    shutil.copytree(CORPUS / 'cases' / case['id'], work)
    result = {'id': case['id'], 'engine': case['engine'], 'expected': case['expect'],
              'source_hashes': {n: sha(work/n) for n in case['files']}, 'steps': []}
    engine = executable(case['engine'], args.texbin)
    if not engine:
        return dict(result, status='blocked', reason='engine not installed')
    result['engine_version'] = run([engine, '--version'], work, env, 10)['output']
    result['engine_sha256'] = sha(Path(engine).resolve())
    for i, action in enumerate(case['workflow']):
        if action == 'tex':
            argv = [engine, '-interaction=nonstopmode', '-halt-on-error', '-no-shell-escape', '-recorder', case['entry']]
        else:
            tool = executable(action, args.texbin)
            if not tool:
                return dict(result, status='blocked', reason=action+' not installed')
            result[action+'_version'] = run([tool, '--version'], work, env, 10)['output']
            argv = [tool, 'main']
        step = run(argv, work, env, args.timeout)
        (work / f'step-{i+1}-{action}.txt').write_text(step.pop('output'))
        result['steps'].append(step)
        if step['returncode'] != 0:
            break
    log = (work/'main.log').read_text(errors='replace') if (work/'main.log').exists() else ''
    outputs = '\n'.join(p.read_text(errors='replace') for p in sorted(work.glob('step-*.txt')))
    text = log+'\n'+outputs
    result['warnings'] = sorted(set(re.findall(r'^.*(?:Warning|Overfull|Underfull|Missing character).*$' ,log,re.M)))
    result['diagnostic_observed'] = case['diagnostic'] in text if case['diagnostic'] else None
    result['dependencies'] = []
    fls = work/'main.fls'
    if fls.exists():
        seen=set()
        for line in fls.read_text(errors='replace').splitlines():
            if not line.startswith('INPUT '): continue
            p = Path(line[6:]); p = p if p.is_absolute() else work/p
            p=p.resolve()
            if p in seen or not p.is_file(): continue
            seen.add(p)
            result['dependencies'].append({'path':str(p), 'sha256':sha(p)})
    pdf = work/'main.pdf'
    if pdf.exists():
        result['pdf_sha256']=sha(pdf)
        result['pdf_bytes']=pdf.stat().st_size
    failed = any(s['returncode'] != 0 for s in result['steps'])
    if any(s.get('timeout') for s in result['steps']):
        result['status']='timeout'
    elif case['expect']=='error':
        result['status']='expected-error' if result['diagnostic_observed'] else 'unexpected-result'
    else:
        unresolved = bool(re.search(r'undefined references|undefined citations|Rerun to get|Please \(re\)run Biber|Missing character:', log))
        result['status']='reference-built' if not failed and pdf.exists() and not unresolved else 'reference-failed'
        if result['status']=='reference-built' and args.render:
            gs=shutil.which('gs')
            if not gs:
                result['raster_status']='blocked: Ghostscript missing'
            else:
                argv=[gs,'-dSAFER','-dBATCH','-dNOPAUSE','-sDEVICE=png16m','-r96','-dTextAlphaBits=4','-dGraphicsAlphaBits=4','-sOutputFile=page-%03d.png','main.pdf']
                step=run(argv,work,env,args.timeout)
                (work/'raster.log').write_text(step.pop('output'))
                result['raster']=step
                result['rasterizer_version']=run([gs,'--version'],work,env,10)['output'].strip()
                result['rasterizer_sha256']=sha(Path(gs).resolve())
                result['pages']=[{'file':p.name,'sha256':sha(p)} for p in sorted(work.glob('page-*.png'))]
                result['raster_status']='rendered' if step['returncode']==0 and result['pages'] else 'failed'
    (work/'reference.json').write_text(json.dumps(result,indent=2)+'\n')
    return result

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output', type=Path)
    p.add_argument('--only', action='append', default=[])
    p.add_argument('--texbin', type=Path, default=Path('/Library/TeX/texbin'))
    p.add_argument('--timeout', type=int, default=60)
    p.add_argument('--render', action='store_true')
    p.add_argument('--validate', action='store_true')
    p.add_argument('--emit-request', metavar='CASE_ID')
    args=p.parse_args()
    manifest=json.loads((CORPUS/'manifest.json').read_text()); validate(manifest)
    if args.emit_request:
        case = next((c for c in manifest['cases'] if c['id'] == args.emit_request), None)
        if case is None: p.error('unknown case ID')
        try:
            for n in case['files']: (CORPUS/'cases'/case['id']/n).read_text(encoding='utf-8')
        except UnicodeDecodeError:
            p.error('binary assets require a project-files adapter; runtime-v1 documents cannot carry PNG bytes')
        print(json.dumps({'protocol_version': 1, 'id': 'extended-'+case['id'], 'type': 'compile',
            'payload': {'project_id': 'extended-'+case['id'], 'revision': 1, 'entry_path': case['entry'],
                'documents': [{'path': n, 'text': (CORPUS/'cases'/case['id']/n).read_text()} for n in case['files']]}}))
        return 0
    if args.validate:
        print(f"Validated {len(manifest['cases'])} projects"); return 0
    if not args.output: p.error('--output is required for generation')
    selected=[c for c in manifest['cases'] if not args.only or c['id'] in args.only]
    if set(args.only)-{c['id'] for c in selected}: p.error('unknown case ID')
    out=args.output.resolve(); out.mkdir(parents=True,exist_ok=False)
    env=os.environ.copy(); env.update(SOURCE_DATE_EPOCH='1789171200',FORCE_SOURCE_DATE='1',TZ='UTC',openin_any='p',openout_any='p')
    cache=out/'tex-cache'; cache.mkdir()
    env['TEXMFVAR']=str(cache); env['TEXMFCACHE']=str(cache)
    env['PATH']=str(args.texbin)+os.pathsep+env.get('PATH','')
    results=[]
    for case in selected:
        result=build(case,out,args,env); results.append(result)
        print(case['id']+': '+result['status'],flush=True)
        (out/'report.json').write_text(json.dumps({'manifest_sha256':sha(CORPUS/'manifest.json'),'source_date_epoch':env['SOURCE_DATE_EPOCH'],'results':results},indent=2)+'\n')
    return int(any(r['status'] not in ('reference-built','expected-error') or r.get('raster_status')=='failed' for r in results))
if __name__=='__main__':
    sys.exit(main())
