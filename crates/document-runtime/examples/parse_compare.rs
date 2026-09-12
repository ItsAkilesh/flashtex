//! Paired parser experiment on one captured compiler JSON frame; no provider calls.
use serde_json::Value;
use std::{hint::black_box, time::Instant};
fn main() {
    let path = std::env::args().nth(1).expect("captured JSON frame path");
    let bytes = std::fs::read(path).unwrap();
    let expected: Value = serde_json::from_slice(&bytes).unwrap();
    for round in 0..20 {
        for optimized in if round % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            let start = Instant::now();
            let value: Value = if optimized {
                serde_json::from_str(std::str::from_utf8(black_box(&bytes)).unwrap()).unwrap()
            } else {
                serde_json::from_slice(black_box(&bytes)).unwrap()
            };
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            assert_eq!(value, expected);
            println!(
                "{}",
                serde_json::json!({"round":round,"utf8_once":optimized,"parse_ms":elapsed,"bytes":bytes.len(),"equal":true})
            );
            black_box(value);
        }
    }
}
