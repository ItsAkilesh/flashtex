#!/usr/bin/env python3
"""Record an offline real-compiler/helper review workflow; never invokes a provider."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--helper', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    env = dict(os.environ)
    env.pop('FLASHTEX_GROK_API_KEY', None)
    def run(binary, request):
        output = subprocess.run([str(binary.resolve())], input=json.dumps(request)+'\n',
                                text=True, capture_output=True, timeout=10, env=env)
        if output.returncode:
            raise RuntimeError(f'{binary.name} failed: {output.returncode}')
        return json.loads(output.stdout)
    text = '\\documentclass{article}\n\\begin{document}\n\\unknowncommand\n\\end{document}\n'
    digest = hashlib.sha256(text.encode()).hexdigest()
    source = dict(project_id='review-fixture',path='main.tex',revision=1,text=text,source_sha256=digest)
    compile_request = dict(protocol_version=1,type='compile',id='fixture-compile',payload=dict(
        project_id='review-fixture',revision=1,entry_path='main.tex',documents=[dict(path='main.tex',revision=1,text=text)]))
    compile_result = run(args.compiler, compile_request)
    assert compile_result['payload']['diagnostics']
    request = dict(operation='prepare',binding=dict(request_id='fixture-compile',project_id='review-fixture',compile_revision=1,
        sources={'main.tex':dict(revision=1,sha256=digest)}),sources=[source],compiler_result=compile_result,
        user_instruction='Explain the unknown command and propose plain text in its place.')
    prepared = run(args.helper,request)
    start = text.encode().index(b'\\unknowncommand')
    proposal = dict(context_id=prepared['payload']['context_id'],explanation='Offline fixture: replace an unknown command with plain text.',
        edits=[dict(location=dict(path='main.tex',start_byte=start,end_byte=start+len(b'\\unknowncommand')),
                    removed_text='\\unknowncommand',replacement='Example text')])
    review_request = dict(request,operation='review',response=proposal,current_sources=[source],explanation_request_id='offline-fixture:1')
    reviewed = run(args.helper,review_request)
    approved_request = dict(review_request,operation='approve',user_approved=True,approved_review_id=reviewed['review_id'])
    approved = run(args.helper,approved_request)
    assert approved['applied'] is False
    assert approved['payload']['group']['expected_sha256']==digest
    artifact = dict(kind='offline_native_review_fixture',provider_called=False,source_mutated=False,
        provenance={name:dict(path=str(path.resolve()),sha256=hashlib.sha256(path.read_bytes()).hexdigest())
                    for name,path in [('compiler',args.compiler),('helper',args.helper)]},
        steps=[dict(request=compile_request,response=compile_result),dict(request=request,response=prepared),
               dict(request=review_request,response=reviewed),dict(request=approved_request,response=approved)])
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(artifact,indent=2,ensure_ascii=False)+'\n')
    print(f'wrote {args.output}; four real subprocess steps, no provider call or apply')

if __name__ == '__main__':
    main()
