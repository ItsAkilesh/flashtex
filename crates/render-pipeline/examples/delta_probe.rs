//! Probe: drive the delta path in-process and print why a delta was or was
//! not emitted for a small document. `cargo run --release --example delta_probe`.
use flashtex_compiler::json;
use flashtex_render_pipeline::{delta, protocol, FontSet, RenderCache, RenderOptions};

fn req_text(id: &str, rev: f64, text: &str, ack: Option<&delta::BaseAck>) -> String {
    let mut payload = json::Value::obj();
    payload.set("project_id", json::str_("delta-gate"));
    payload.set("revision", json::num(rev));
    payload.set("entry_path", json::str_("main.tex"));
    let mut doc = json::Value::obj();
    doc.set("path", json::str_("main.tex"));
    doc.set("text", json::str_(text));
    payload.set("documents", json::Value::Arr(vec![doc]));
    payload.set(
        "layout_capabilities",
        json::Value::Arr(["rules-v1", "font-hints-v1", "display-list-v2", delta::CAP_DELTA].into_iter().map(json::str_).collect()),
    );
    let _ = ack;
    let mut v = json::Value::obj();
    v.set("protocol_version", json::num(1.0));
    v.set("id", json::str_(id));
    v.set("type", json::str_("compile"));
    v.set("payload", payload);
    json::write(&v)
}

fn main() {
    let fonts = FontSet::with_default_dirs(&[]);
    let options = RenderOptions::default();
    let cache = RenderCache::new();
    let sections: usize = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(3);
    if let Some(path) = std::env::args().nth(2) {
        // digest probe for a text file: print the snapshot digests the producer computes
        let text = std::fs::read_to_string(&path).expect("text file");
        let r0 = protocol::handle_line(&req_text("probe-1", 1.0, &text, None), &fonts, &options, Some(&cache));
        let _ = r0;
        let s = cache.delta_snapshot().expect("snapshot");
        println!("list_digest {}", flashtex_font_engine::sha256::hex(&s.list_digest));
        for (i, d) in s.page_digests.iter().enumerate() {
            println!("page {} digest {} bytes {}", i + 1, flashtex_font_engine::sha256::hex(d), s.page_bytes[i]);
        }
        println!("header_digest {}", flashtex_font_engine::sha256::hex(&delta::header_digest(&s.list)));
        println!("header_canon {}", flashtex_font_engine::sha256::hex(&delta::header_canon(&s.list)));
        return;
    }
    let mut t0 = String::from("\\begin{document}\n");
    for i in 0..sections {
        t0.push_str(&format!("\\section{{Part {i}}}\n"));
        for j in 0..4 {
            t0.push_str(&format!("Paragraph {i}.{j}. The quick brown fox jumps over the lazy dog while the patient owl watches from an old oak branch and counts every leaf that falls into the quiet river below; then more text so that the paragraph wraps onto several lines of the page.\n\n"));
        }
        if i % 3 == 2 {
            t0.push_str("\\newpage\n");
        }
    }
    t0.push_str("\\end{document}\n");
    let t1 = t0.replacen("Paragraph 0.0.", "Paragraph 0.0 edited.", 1);
    let req = |id: &str, rev: f64, text: &str, ack: Option<&delta::BaseAck>| {
        let mut payload = json::Value::obj();
        payload.set("project_id", json::str_("probe"));
        payload.set("revision", json::num(rev));
        payload.set("entry_path", json::str_("main.tex"));
        let mut doc = json::Value::obj();
        doc.set("path", json::str_("main.tex"));
        doc.set("text", json::str_(text));
        payload.set("documents", json::Value::Arr(vec![doc]));
        payload.set(
            "layout_capabilities",
            json::Value::Arr(["rules-v1", "font-hints-v1", "display-list-v2", delta::CAP_DELTA].into_iter().map(json::str_).collect()),
        );
        if let Some(a) = ack {
            let mut b = json::Value::obj();
            b.set("request_id", json::str_(a.request_id.clone()));
            b.set("project_id", json::str_(a.project_id.clone()));
            b.set("revision", json::num(a.revision as f64));
            b.set("page_count", json::num(a.page_count as f64));
            b.set("list_digest", json::str_(a.list_digest.clone()));
            payload.set("display_list_base", b);
        }
        let mut v = json::Value::obj();
        v.set("protocol_version", json::num(1.0));
        v.set("id", json::str_(id));
        v.set("type", json::str_("compile"));
        v.set("payload", payload);
        json::write(&v)
    };
    let r0 = protocol::handle_line(&req("a", 1.0, &t0, None), &fonts, &options, Some(&cache));
    println!("r0: {} extra, line head {}", r0.extra_lines.len(), &r0.line[..r0.line.len().min(160)]);
    let Some(s0) = cache.delta_snapshot() else {
        println!("no snapshot retained after r0");
        return;
    };
    println!("snapshot: {} pages, page_bytes {:?}", s0.list.pages.len(), s0.page_bytes);
    let ack = delta::BaseAck {
        request_id: s0.request_id.clone(),
        project_id: s0.list.project_id.clone(),
        revision: s0.list.revision,
        page_count: s0.list.pages.len(),
        list_digest: flashtex_font_engine::sha256::hex(&s0.list_digest),
    };
    let r1 = protocol::handle_line(&req("b", 2.0, &t1, Some(&ack)), &fonts, &options, Some(&cache));
    let rendered = r1.rendered.as_ref().unwrap();
    match delta::build(&s0, &ack, &rendered.v2, &[("main.tex".to_string(), t1.clone())]) {
        Ok(d) => {
            let dl = json::write(&d.to_json("b", &rendered.v2));
            let full = delta::full_line_bytes(&rendered.v2, "b", &rendered.v2.required_features(), &d.page_bytes);
            println!("delta built: changed {:?} of {} pages; delta line {} bytes; full {:?}; policy ok {}", d.changed, d.page_count, dl.len(), full, full.map_or(false, |f| dl.len() * delta::POLICY_DEN <= f * delta::POLICY_NUM));
        }
        Err(e) => println!("no delta: {e:?}"),
    }
    println!("r1: {} extra, echo line head {}", r1.extra_lines.len(), &r1.line[..r1.line.len().min(200)]);
}
