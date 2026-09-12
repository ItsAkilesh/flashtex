#!/usr/bin/env python3
"""Instrument an isolated pinned binder copy; never changes production sources.
Allocation bytes are cumulative requested capacity (including realloc requests),
not live residency or RSS. Timing mode disables allocation counter updates.
"""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import statistics
import subprocess
import tarfile
import tempfile

BASE = "3040a0ce37256999db73ac73fd7b9df693aad1a1"
CRATES = ["rendering-core", "font-resources", "font-engine", "pdf", "project-files", "paragraph-layout", "math-layout"]
PROFILE = r'''
#[doc(hidden)]
pub mod attribution {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
    use std::time::Instant;
    static COUNT: AtomicBool = AtomicBool::new(false);
    static CALLS: AtomicU64 = AtomicU64::new(0);
    static BYTES: AtomicU64 = AtomicU64::new(0);
    static CASE: AtomicU64 = AtomicU64::new(0);
    static SAMPLE: AtomicU64 = AtomicU64::new(0);
    struct Counter;
    unsafe impl GlobalAlloc for Counter {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            if COUNT.load(Relaxed) { CALLS.fetch_add(1, Relaxed); BYTES.fetch_add(layout.size() as u64, Relaxed); }
            System.alloc(layout)
        }
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            if COUNT.load(Relaxed) { CALLS.fetch_add(1, Relaxed); BYTES.fetch_add(layout.size() as u64, Relaxed); }
            System.alloc_zeroed(layout)
        }
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            if COUNT.load(Relaxed) { CALLS.fetch_add(1, Relaxed); BYTES.fetch_add(size as u64, Relaxed); }
            System.realloc(ptr, layout, size)
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) { System.dealloc(ptr, layout) }
    }
    #[global_allocator]
    static ALLOCATOR: Counter = Counter;
    pub fn context(case: u64, sample: u64, count: bool) {
        CASE.store(case, Relaxed); SAMPLE.store(sample, Relaxed); COUNT.store(count, Relaxed);
    }
    pub struct Mark(Instant, u64, u64);
    #[derive(serde::Serialize)]
    pub struct Phase { name: &'static str, ns: u128, allocation_calls: u64, requested_bytes: u64 }
    pub fn begin() -> Mark { Mark(Instant::now(), CALLS.load(Relaxed), BYTES.load(Relaxed)) }
    pub fn finish(mark: Mark, name: &'static str) -> Phase {
        Phase { name, ns: mark.0.elapsed().as_nanos(), allocation_calls: CALLS.load(Relaxed)-mark.1, requested_bytes: BYTES.load(Relaxed)-mark.2 }
    }
    pub fn emit(phases: &[Phase]) {
        eprintln!("BIND_PROFILE {}", serde_json::json!({"case":CASE.load(Relaxed),"sample":SAMPLE.load(Relaxed),"counted":COUNT.load(Relaxed),"phases":phases}));
    }
}
'''
TEST = r'''
#[test]
#[ignore = "isolated instrumented attribution only; no native performance claim"]
fn attribution_only() {
    use flashtex_rendering_core::helper_candidate::attribution;
    let counted = std::env::var("PROFILE_COUNT_ALLOCATIONS").unwrap() == "1";
    for (index, (raw, meta)) in [
        (include_bytes!("fixtures/helper-raw-f5524794/step-0.candidate.jsonl").as_slice(),include_bytes!("fixtures/helper-raw-f5524794/step-0.metadata.json").as_slice()),
        (include_bytes!("fixtures/helper-raw-f5524794/step-1.candidate.jsonl").as_slice(),include_bytes!("fixtures/helper-raw-f5524794/step-1.metadata.json").as_slice()),
        (include_bytes!("fixtures/helper-raw-f5524794/step-2.candidate.jsonl").as_slice(),include_bytes!("fixtures/helper-raw-f5524794/step-2.metadata.json").as_slice()),
    ].into_iter().enumerate() {
        let event: Value=serde_json::from_slice(raw).unwrap();
        let metadata: Value=serde_json::from_slice(meta).unwrap(); let c=&metadata["current"];
        let current=CurrentHelper {
            session_id:c["session_id"].as_str().unwrap().into(),project_id:c["project_id"].as_str().unwrap().into(),request_id:c["request_id"].as_str().unwrap().into(),
            compile_revision:c["compile_revision"].as_u64().unwrap(),membership_generation:c["membership_generation"].as_u64().unwrap(),
            sources:c["sources"].as_object().unwrap().iter().map(|(p,s)|(p.clone(),CurrentSource{editor_revision:s["editor_revision"].as_u64().unwrap(),text:s["text"].as_str().unwrap().into()})).collect(),
        };
        let resources=resources(&event["payload"]["display_list"]); let result=serde_json::to_vec(&metadata["result"]).unwrap(); let caps=caps();
        for sample in 0..23 {
            attribution::context(index as u64,sample,counted);
            let bound=bind(raw,&result,&current,&caps,&resources).unwrap();
            let mark=attribution::begin(); drop(bound);
            attribution::emit(&[attribution::finish(mark,"bound_drop")]);
        }
    }
}
'''

def command(args, cwd, env=None, timeout=240):
    return subprocess.run(args, cwd=cwd, env=env, capture_output=True, check=True, timeout=timeout)

def sha(data):
    return hashlib.sha256(data).hexdigest()

def replace_once(text, old, new):
    if text.count(old) != 1:
        raise RuntimeError("instrumentation anchor drift: " + old[:70])
    return text.replace(old,new,1)

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--report',type=Path,required=True); args=ap.parse_args()
    root=Path(__file__).resolve().parents[3]
    base=command(['git','rev-parse',BASE],root).stdout.decode().strip()
    archive=command(['git','archive',base,*['crates/'+c for c in CRATES]],root).stdout
    env=dict(os.environ,CARGO_NET_OFFLINE='true',CARGO_BUILD_JOBS='2',CARGO_INCREMENTAL='0')
    (root/'target').mkdir(exist_ok=True)
    env['CARGO_TARGET_DIR']=str(root/'target/binder-attribution')
    with tempfile.TemporaryDirectory(prefix='binder-attribution-',dir=root/'target') as tmp:
        tmp=Path(tmp)
        with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
            tar.extractall(tmp,filter='data')
        p=tmp/'crates/rendering-core/src/helper_candidate.rs'; original=p.read_text(); s=original
        s=replace_once(s,'    let _: serde_json::Value =', '    let mut phases = Vec::with_capacity(6);\n    let mark = attribution::begin();\n    let _: serde_json::Value =')
        s=replace_once(s,'    let value: RawHelperEvent =', '    phases.push(attribution::finish(mark,"value_preflight_parse_drop"));\n    let mark = attribution::begin();\n    let value: RawHelperEvent =')
        s=replace_once(s,'    let p = &value.payload;', '    phases.push(attribution::finish(mark,"typed_wrapper_decode"));\n    let mark = attribution::begin();\n    let p = &value.payload;')
        s=replace_once(s,'    let bytes = p.display_list.get().as_bytes();', '    phases.push(attribution::finish(mark,"policy_source_snapshot"));\n    let mark = attribution::begin();\n    let bytes = p.display_list.get().as_bytes();')
        s=replace_once(s,'    let display = PipelineCff::bind(bytes, capabilities, &documents, resources)?;', '    phases.push(attribution::finish(mark,"pipeline_pair"));\n    let mark = attribution::begin();\n    let display = PipelineCff::bind(bytes, capabilities, &documents, resources)?;\n    phases.push(attribution::finish(mark,"pipeline_resource_bind"));\n    let mark = attribution::begin();\n    drop(value);\n    phases.push(attribution::finish(mark,"typed_wrapper_drop"));\n    attribution::emit(&phases);')
        s+='\n'+PROFILE; p.write_text(s)
        t=tmp/'crates/rendering-core/tests/helper_candidate.rs'; t.write_text(t.read_text()+TEST)
        cargo=['cargo','test','--release','--manifest-path','crates/rendering-core/Cargo.toml','--test','helper_candidate']
        command(cargo+['--no-run'],tmp,env,timeout=300)
        records=[]
        for count in ['0','1']:
            result=command(cargo+['--','--ignored','--exact','attribution_only','--nocapture','--test-threads=1'],tmp,dict(env,PROFILE_COUNT_ALLOCATIONS=count))
            for line in result.stderr.decode().splitlines():
                if line.startswith('BIND_PROFILE '): records.append(json.loads(line[len('BIND_PROFILE '):]))
        summary=[]
        for case in range(3):
            for name in ['value_preflight_parse_drop','typed_wrapper_decode','policy_source_snapshot','pipeline_pair','pipeline_resource_bind','typed_wrapper_drop','bound_drop']:
                timing=[p for r in records if r['case']==case and not r['counted'] and r['sample']>=3 for p in r['phases'] if p['name']==name]
                alloc=[p for r in records if r['case']==case and r['counted'] and r['sample']>=3 for p in r['phases'] if p['name']==name]
                if len(timing)!=20 or len(alloc)!=20: raise RuntimeError('incomplete samples')
                summary.append({'case':case,'phase':name,'samples':20,'median_ns':statistics.median(p['ns'] for p in timing),'min_ns':min(p['ns'] for p in timing),'max_ns':max(p['ns'] for p in timing),'median_allocation_calls':statistics.median(p['allocation_calls'] for p in alloc),'median_requested_bytes':statistics.median(p['requested_bytes'] for p in alloc)})
        report={'format':'flashtex-binder-attribution-v1','base_commit':base,'original_helper_source_sha256':sha(original.encode()),'instrumented_helper_source_sha256':sha(s.encode()),'script_sha256':sha(Path(__file__).read_bytes()),'archive_sha256':sha(archive),'rustc':command(['rustc','--version'],root).stdout.decode().strip(),'cargo':command(['cargo','--version'],root).stdout.decode().strip(),'scope':'Instrumented release binder phases on existing actual raw helper captures; 3 warmups +20samples per mode. Timing counter updates disabled. Allocations are requested capacity, not retained memory/RSS. Phase timers exclude profiling output; no native or production speedup claim.','summary':summary,'samples':records}
        args.report.parent.mkdir(parents=True,exist_ok=True); args.report.write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps({'report':str(args.report),'summaries':summary}))

if __name__=='__main__': main()
