import pathlib,subprocess,json,os,hashlib
p=pathlib.Path(__file__).parent
cases={'five':r'\text{(a)}+\text{(b)}+\text{(c)}+\text{(d)}+\text{and}', 'scripts':r'\text{and}_{i}^{2}', 'space':r'\text{a b}', 'escaped':r'\text{\{x\}}', 'ligature':r'\text{ffi}', 'comment':"\\text% before\n{and}"}
summary=[]
for name,math in cases.items():
 source='\\documentclass[12pt]{article}\n\\begin{document}\n$'+math+'$\n\\end{document}\n'
 request={'protocol_version':1,'id':name,'type':'compile','payload':{'project_id':name,'revision':1,'entry_path':'main.tex','documents':[{'path':'main.tex','text':source}]}}
 d=p/'cases'/name;d.mkdir(parents=True,exist_ok=True); wire=json.dumps(request)+'\n';(d/'request.jsonl').write_text(wire)
 env=dict(os.environ,FLASHTEX_TFM_DIRS=str(p/'metrics'),FLASHTEX_FONT_DIRS=str(p/'fonts'))
 r=subprocess.run([str(p/'producer/target/release/flashtex-render'),'--font-dir',str(p/'fonts'),'--v2',str(d/'display.json')],input=wire,text=True,capture_output=True,env=env,timeout=30)
 (d/'stdout.jsonl').write_text(r.stdout);(d/'stderr.txt').write_text(r.stderr)
 summary.append({'case':name,'returncode':r.returncode,'stdout_bytes':len(r.stdout),'files':{f.name:hashlib.sha256(f.read_bytes()).hexdigest() for f in d.iterdir() if f.is_file()}})
(p/'summary.json').write_text(json.dumps(summary,indent=2)+'\n'); print(json.dumps(summary,indent=2))
