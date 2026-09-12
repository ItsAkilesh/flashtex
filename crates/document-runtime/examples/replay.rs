//! Replay complete runtime requests against explicitly supplied original compiler.
use flashtex_document_runtime::{Document, Event, Limits, Request, Session};
use serde_json::{json, Value};
use std::{
    io::{self, BufRead, Read},
    path::Path,
    thread,
    time::{Duration, Instant},
};

fn response(session: &mut Session) -> Result<(Value, [f64; 3]), String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let mut events = session.poll();
        if events.len() > 1 {
            return Err("multiple events for sequential replay".into());
        }
        if let Some(event) = events.pop() {
            match event {
                Event::Preview {
                    result,
                    queue_ms,
                    compiler_ms,
                    total_ms,
                    ..
                } => return Ok((result, [queue_ms, compiler_ms, total_ms])),
                Event::Failed { reason, .. } => return Err(reason),
                other => return Err(format!("unexpected replay event: {other:?}")),
            }
        }
        if Instant::now() >= deadline {
            return Err("replay deadline exceeded".into());
        }
        thread::sleep(Duration::from_micros(100));
    }
}
fn mismatch_class(persistent: &Value, fresh: &Value) -> &'static str {
    if persistent == fresh {
        "none"
    } else if persistent["payload"]["status"] != fresh["payload"]["status"] {
        "status"
    } else if persistent["payload"]["diagnostics"] != fresh["payload"]["diagnostics"] {
        "diagnostics"
    } else if persistent["payload"]["pages"] != fresh["payload"]["pages"] {
        "positioned_pages"
    } else {
        "envelope_or_other_payload"
    }
}
fn run(binary: &Path, limits: Limits) -> Result<(), String> {
    let mut warm = Session::spawn(binary, limits.clone())?;
    let mut samples = Vec::new();
    let mut input = io::stdin().lock();
    for index in 0..=10000 {
        if index >= 10000 {
            return Err("replay capped at 10000 edits".into());
        }
        let mut line = String::new();
        let read = input
            .by_ref()
            .take(limits.max_frame as u64 + 1)
            .read_line(&mut line)
            .map_err(|e| e.to_string())?;
        if read == 0 {
            break;
        }
        if line.len() > limits.max_frame {
            return Err("oversized replay line".into());
        }
        let input: Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;
        let p = &input["payload"];
        let documents: Vec<Document> =
            serde_json::from_value(p["documents"].clone()).map_err(|e| e.to_string())?;
        let string = |v: &Value| {
            v.as_str()
                .map(str::to_owned)
                .ok_or("missing replay string".to_owned())
        };
        let request = Request {
            id: string(&input["id"])?,
            project_id: string(&p["project_id"])?,
            revision: p["revision"].as_u64().ok_or("missing revision")?,
            entry_path: string(&p["entry_path"])?,
            documents,
        };
        let id = request.id.clone();
        let warm_started = Instant::now();
        warm.submit(request.clone())?;
        let (persistent, timing) = response(&mut warm)?;
        let warm_observed_ms = warm_started.elapsed().as_secs_f64() * 1000.0;
        let cold_started = Instant::now();
        let mut cold = Session::spawn(binary, limits.clone())?;
        cold.submit(request)?;
        let (fresh, _) = response(&mut cold)?;
        let fresh_observed_ms = cold_started.elapsed().as_secs_f64() * 1000.0;
        let equal = persistent == fresh;
        println!(
            "{}",
            json!({"type":"sample", "id":id, "exact_json_equal":equal,
            "mismatch_class":mismatch_class(&persistent, &fresh),
            "compiler_status":persistent["payload"]["status"],
            "pages":persistent["payload"]["pages"].as_array().map(Vec::len),
            "diagnostics":persistent["payload"]["diagnostics"].as_array().map(Vec::len),
            "warm_queue_ms":timing[0], "warm_compiler_transport_poll_ms":timing[1],
            "warm_total_ms":timing[2], "warm_call_to_result_ms":warm_observed_ms,
            "fresh_launch_to_result_ms":fresh_observed_ms})
        );
        if !equal {
            return Err(format!("persistent/fresh result mismatch for {id}"));
        }
        samples.push(warm_observed_ms);
    }
    if samples.is_empty() {
        return Err("no replay edits supplied".into());
    }
    samples.sort_by(f64::total_cmp);
    let percentile = |p: usize| samples[(samples.len() * p).div_ceil(100).saturating_sub(1)];
    println!(
        "{}",
        json!({"type":"summary", "max_frame_bytes":limits.max_frame,"edits":samples.len(), "p50_ms":percentile(50),
        "p95_ms":percentile(95), "p99_ms":percentile(99), "max_ms":samples.last(),
        "measurement":"warm submit call through validated positioned result; request cloning/encoding, compiler, transport, JSON validation and polling combined",
        "native_paint_measured":false, "reference_pdf_measured":false})
    );
    Ok(())
}
fn main() {
    let result = std::env::args_os()
        .nth(1)
        .ok_or("usage: replay /absolute/original/compiler < edits.jsonl".to_owned())
        .and_then(|binary| {
            let mut limits = Limits::default();
            if let Some(value) = std::env::args().nth(2) {
                limits.max_frame = value.parse::<usize>().map_err(|_| "invalid frame bytes")?;
            }
            run(Path::new(&binary), limits)
        });
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
