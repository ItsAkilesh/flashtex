#!/usr/bin/env python3
"""Import pinned original helper capture; local licensed font must match wire SHA."""
import subprocess,json,hashlib,pathlib
sha=subprocess.check_output(['git','rev-parse','eca6ab2582e18dc65ef759b4d39f8ec6502df777'],text=True).strip();base='crates/preview-controller/benchmarks/display-helper-multidoc-10pt/'
def read(p):return subprocess.check_output(['git','show',sha+':'+base+p])
def digest(b):return hashlib.sha256(b).hexdigest()
files=subprocess.check_output(['git','ls-tree','-r','--name-only',sha,base],text=True).splitlines()
inputs={};outputs={}
for case in ['control','verified']:
 inputs[case]=[];outputs[case]=[]
 for f in files:
  if '/'+case+'/' in f and f.endswith('.input.jsonl'):inputs[case]+=read(f[len(base):]).splitlines()
  if '/'+case+'/' in f and f.endswith('.output.jsonl') and '/producer-' in f:outputs[case]+=read(f[len(base):]).splitlines()
 assert len(inputs[case])==6
assert inputs['control']==inputs['verified']
for case in ['control','verified']:
 results=[json.loads(x) for x in outputs[case] if json.loads(x)['type']=='compile_result'];assert len(results)==6
 assert all(x['payload']['status']==('ok' if case=='verified' else 'recovered') for x in results)
 if case=='verified':assert all(x['payload']['diagnostics']==[] for x in results)
helper=read('verified/helper.output.jsonl');raw=next(l for l in helper.splitlines(keepends=True) if json.loads(l).get('payload',{}).get('kind')=='display_candidate' and json.loads(l)['payload']['compile_revision']==6)
e=json.loads(raw);p=e['payload'];rid=p['request_id'];request=next(x for x in inputs['verified'] if json.loads(x)['id']==rid);result=next(x for x in outputs['verified'] if json.loads(x)['id']==rid and json.loads(x)['type']=='compile_result');body=next(x for x in outputs['verified'] if json.loads(x)['id']==rid and json.loads(x)['type']=='display_list');assert raw.endswith(b'"display_list":'+body+b'}}\n')
q=json.loads(request);current=dict(session_id=e['session_id'],project_id=p['project_id'],request_id=rid,compile_revision=6,membership_generation=p['membership_generation'],sources={d['path']:dict(editor_revision=p['source_versions'][d['path']],text=d['text']) for d in q['payload']['documents']})
assert current['sources'].keys()=={'main.tex','chapter.tex','refs.bib'}
snapshot=read('verified/helper.snapshot.json');assert json.loads(snapshot)['document_kinds']['refs.bib']=='bibliography'
d=pathlib.Path('crates/rendering-core/tests/fixtures/helper-multidoc-eca6ab25');d.mkdir(exist_ok=True)
artifacts={'candidate.jsonl':raw,'producer.jsonl':body+b'\n','request.jsonl':request+b'\n','result.jsonl':result+b'\n','snapshot.json':snapshot,'metadata.json':(json.dumps({'current':current,'result':json.loads(result)},indent=2)+'\n').encode()}
font=pathlib.Path('/tmp/flashtex-render-reference-stage/fonts/lmroman10-regular.otf').read_bytes();assert digest(font)==p['display_list']['payload']['fonts'][0]['sha256'];artifacts['lmroman10-regular.otf']=font
for n,b in artifacts.items():(d/n).write_bytes(b)
manifest=dict(upstream=sha,upstream_directory=base,original_recovered_capture='e680d2ef51d3fc6cc9e38f55fdbbc0f43f1aa183',six_control_verified_request_bytes_identical=True,control_six_recovered=True,verified_six_ok_zero_diagnostics=True,raw_nested_body_exact=True,source_versions=p['source_versions'],artifacts={n:digest(b) for n,b in artifacts.items()},scope='Original candidate/request/result/body bytes; metadata is derived caller test input only. Snapshot records bibliography kind, not rendered bibliography. Font bytes match actual emitted identity; existing GUST license applies. No native or reference parity claim.')
(d/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(manifest,indent=2))
