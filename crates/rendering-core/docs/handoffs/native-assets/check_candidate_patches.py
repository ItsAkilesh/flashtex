import pathlib,tempfile,subprocess,json,os,shutil,hashlib
repo=pathlib.Path.cwd();out=repo/'crates/rendering-core/docs/handoffs/native-assets';meta=json.load(open(out/'patch-manifest.json'));observed={}
with tempfile.TemporaryDirectory(prefix='flashtex-native-patch-check-') as tmp:
 scratch=pathlib.Path(tmp);subprocess.run(['git','init','-q',str(scratch)],check=True)
 for patch,base,path in [('packaging.patch',meta['packaging_base'],'apps/mac/scripts/make-app.sh'),('producer-discovery.patch',meta['producer_base'],'crates/render-pipeline/src/fonts.rs')]:
  target=scratch/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(subprocess.check_output(['git','show',base+':'+path]));subprocess.run(['git','apply','--check',str(out/patch)],cwd=scratch,check=True);subprocess.run(['git','apply',str(out/patch)],cwd=scratch,check=True);observed[patch]={'git_apply_check':True,'patched_source_sha256':hashlib.sha256(target.read_bytes()).hexdigest()}
 subprocess.run(['bash','-n',str(scratch/'apps/mac/scripts/make-app.sh')],check=True);observed['packaging_bash_syntax']=True
 source=(scratch/'crates/render-pipeline/src/fonts.rs').read_text();f=source[source.index('pub fn default_tfm_dirs()'):];f=f[:f.index('\n}\n')];assert f.index('FLASHTEX_TFM_DIRS')<f.index('let bundled =')<f.index('for d in default_font_dirs()');assert 'set_var' not in f;observed['explicit_tfm_environment_precedes_bundle_without_mutation']=True
 resources=scratch/'app/Contents/Resources';metrics=scratch/'official-texmf';manifest=json.load(open(out/'manifest.json'))
 fonts={'lmroman10-regular.otf':repo/'crates/rendering-core/tests/fixtures/helper-multidoc-eca6ab25/lmroman10-regular.otf','lmroman12-regular.otf':repo/'crates/rendering-core/tests/fixtures/original-reference/lmroman12-regular.otf','latinmodern-math.otf':repo/'crates/rendering-core/tests/fixtures/math-reference/latinmodern-math.otf'}
 for e in manifest['required_for_existing_fixtures']:
  n=pathlib.Path(e['path']).name
  if e['path'].startswith('Fonts/'):
   src=fonts[n];dest=resources/e['path']
  else:
   src=pathlib.Path('/tmp/flashtex-lm-tfm-0or97xdj/v2.004')/('LICENSE' if n=='GUST-FONT-LICENSE.TXT' else n);dest=metrics/e['path'][len('texmf/'):]
  b=src.read_bytes();assert hashlib.sha256(b).hexdigest()==e['sha256'];dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(b)
 text=(scratch/'apps/mac/scripts/make-app.sh').read_text();block=text[text.index('# --- Pinned bundle metrics'):text.index('# --- End pinned bundle metrics')];stage=scratch/'isolated-stage.sh';stage.write_text('set -eu\ndie() { echo "$*" >&2; exit 1; }\n'+block)
 env=dict(os.environ,REPO_ROOT=str(repo),FLASHTEX_BUNDLE_TEXMF_ROOT=str(metrics),RESOURCES_DIR=str(resources));result=subprocess.run(['bash',str(stage)],env=env,capture_output=True,timeout=10);assert result.returncode==0,result.stderr
 report=json.load(open(resources/'resource-coverage.json'));assert report['status']=='verified';observed['isolated_packaging_block']={'exit_code':0,'verified_resources':len(report['resources']),'native_discovery_executed':False}
 (metrics/'fonts/tfm/public/lm/ec-lmr10.tfm').unlink();result=subprocess.run(['bash',str(stage)],env=env,capture_output=True,timeout=10);assert result.returncode!=0;observed['missing_source_10pt_block_refusal']={'exit_code':result.returncode,'diagnostic':result.stderr.decode().strip()}
observed['scope']='git-apply and shell staging block only in disposable scratch; no native build/sign/launch, no producer execution or authoritative peer edits';observed['patches']=meta['patches'];(out/'patch-check-evidence.json').write_text(json.dumps(observed,indent=2)+'\n');print(json.dumps(observed,indent=2))
