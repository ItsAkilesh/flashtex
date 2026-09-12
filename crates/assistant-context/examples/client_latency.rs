//! Local helper roundtrip benchmark; no provider calls or native paint claims.
#[cfg(unix)]
fn main() {
    use flashtex_assistant_context::{CompileBinding, SessionClient};
    use flashtex_edit_ledger::Document;
    use serde_json::json;
    use std::{
        path::PathBuf,
        time::{Duration, Instant},
    };
    let executable = PathBuf::from(std::env::args().nth(1).expect("helper executable"));
    let mut client = SessionClient::spawn(&executable, "latency_experiment").unwrap();
    for size in [5_000, 50_000, 500_000, 1_000_000] {
        let docs = vec![Document::new("p".into(), "main.tex".into(), 1, "a".repeat(size)).unwrap()];
        let binding = CompileBinding::capture("compile", "p", 1, &docs).unwrap();
        let input = json!({"operation":"prepare","binding":binding,"sources":docs,"compiler_result":{"protocol_version":1,"type":"compile_result","id":"compile","payload":{"project_id":"p","revision":1,"status":"ok","pages":[],"diagnostics":[]}},"user_instruction":"Explain","related_paths":["main.tex"]});
        let mut times = Vec::new();
        for _ in 0..5 {
            let action = json!({"operation":"submit","input":input,"timeout_ms":10000});
            let start = Instant::now();
            let reply = client.call(action, Duration::from_secs(5)).unwrap();
            times.push(start.elapsed().as_secs_f64() * 1000.0);
            let id = reply["result"]["request_id"]
                .as_str()
                .expect("submitted request");
            client
                .call(
                    json!({"operation":"cancel","request_id":id}),
                    Duration::from_secs(1),
                )
                .unwrap();
        }
        times.sort_by(f64::total_cmp);
        println!(
            "source_bytes={size} samples=5 median_ms={:.3} max_ms={:.3}",
            times[2], times[4]
        );
    }
}
#[cfg(not(unix))]
fn main() {
    eprintln!("Unix client required");
}
