//! `display-list-v2-delta` producer gate (proposal r5 §10.1, isolated):
//! over the `tests/incremental.rs` edit script a delta-mode worker (real
//! `protocol::handle_line`, persistent `RenderCache`) is driven by a reference
//! consumer that acknowledges its installed base; every reply's sibling is
//! either the unchanged full line or a `display_list_delta` whose
//! reconstruction serialises BYTE-IDENTICALLY to a fresh full compile's line.
//! `page_bytes` are checked against the real writer's page objects inside the
//! fresh line (raw ranges), not against any estimate.

mod common;

use common::*;
use flashtex_compiler::json::{self, Value};
use flashtex_render_pipeline::delta::{self, BaseAck, Delta, Refusal, Snapshot};
use flashtex_render_pipeline::display::DisplayList;
use flashtex_render_pipeline::{protocol, FontSet, RenderCache, RenderOptions};

const PARA: &str = "The quick brown fox jumps over the lazy dog while the patient owl watches from an old oak \
branch and counts every leaf that falls into the quiet river below. A \\textbf{bold} word, an \\emph{emphasised} \
one, the UTF-8 word caf\\'e, ``quotes'' --- and inline math $x_{i}^{2} + \\frac{a}{b} = \\sqrt{z}$ inside the \
sentence; then more text so that the paragraph wraps onto several lines of the page.";

fn document(sections: usize) -> String {
    let mut s = String::from("\\begin{document}\n");
    for i in 0..sections {
        s.push_str(&format!("\\section{{Part {i}}}\\label{{s{i}}}\n"));
        for j in 0..4 {
            s.push_str(&format!("Paragraph {i}.{j} of part \\ref{{s{i}}} on page \\pageref{{s{i}}}. {PARA}\n\n"));
        }
        s.push_str("\\[\n\\sum_{k=0}^{n} k^{2} = \\frac{n(n+1)(2n+1)}{6}\n\\]\nAfter the display the paragraph goes on.\n\n");
        if i % 3 == 2 {
            s.push_str("\\newpage\n");
        }
    }
    s.push_str("\\end{document}\n");
    s
}

fn edited(base: &str, i: usize) -> String {
    let mut s = base.to_string();
    let body_start = s.find("\\begin{document}").unwrap() + 16;
    let span = s.len() - body_start - 20;
    let k = body_start + (i * 7919) % span;
    let k = s[..k].rfind(' ').unwrap_or(body_start);
    match i % 5 {
        0 => s.insert_str(k, &format!(" inserted{i}")),
        1 => {
            let end = s[k + 1..].find(' ').map(|e| k + 1 + e).unwrap_or(k + 1);
            if !s[k..end].contains('\\') && !s[k..end].contains('$') && !s[k..end].contains('\n') {
                s.replace_range(k..end, "");
            } else {
                s.insert_str(k, " x");
            }
        }
        2 => s = s.replace("k^{2} = ", &format!("k^{{{}}} = ", 2 + i % 3)),
        3 => {
            if let Some(p) = s.find("Paragraph 1.2") {
                if let Some(e) = s[p..].find("\n\n") {
                    s.replace_range(p..p + e + 2, "");
                }
            }
        }
        _ => s.insert_str(k, &format!(" {}", "word ".repeat(1 + i % 4))),
    }
    s
}

fn request(id: &str, revision: usize, text: &str, delta: bool, ack: Option<&BaseAck>) -> String {
    let mut caps = vec!["rules-v1", "font-hints-v1", "display-list-v2"];
    if delta {
        caps.push(delta::CAP_DELTA);
    }
    let mut payload = Value::obj();
    payload.set("project_id", json::str_("gate"));
    payload.set("revision", json::num(revision as f64));
    payload.set("entry_path", json::str_("main.tex"));
    let mut doc = Value::obj();
    doc.set("path", json::str_("main.tex"));
    doc.set("text", json::str_(text));
    payload.set("documents", Value::Arr(vec![doc]));
    payload.set("layout_capabilities", Value::Arr(caps.into_iter().map(json::str_).collect()));
    if let Some(a) = ack {
        let mut b = Value::obj();
        b.set("request_id", json::str_(a.request_id.clone()));
        b.set("project_id", json::str_(a.project_id.clone()));
        b.set("revision", json::num(a.revision as f64));
        b.set("page_count", json::num(a.page_count as f64));
        b.set("list_digest", json::str_(a.list_digest.clone()));
        payload.set("display_list_base", b);
    }
    let mut v = Value::obj();
    v.set("protocol_version", json::num(1.0));
    v.set("id", json::str_(id));
    v.set("type", json::str_("compile"));
    v.set("payload", payload);
    json::write(&v)
}

/// Byte ranges of the elements of the JSON array under `key` at object
/// depth `depth` (1 = top level, 2 = payload), by structural scan (strings
/// and escapes respected). Used to read the REAL writer's page objects.
fn array_elements(line: &str, key: &str, depth: usize) -> Vec<(usize, usize)> {
    let b = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    let mut d = 0usize;
    let mut last_key: Option<String> = None;
    while i < b.len() {
        match b[i] {
            b'"' => {
                let start = i;
                i += 1;
                while b[i] != b'"' {
                    if b[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                let s = &line[start + 1..i];
                if b.get(i + 1) == Some(&b':') {
                    last_key = Some(s.to_string());
                }
                i += 1;
            }
            b'{' => {
                d += 1;
                i += 1;
            }
            b'}' => {
                d -= 1;
                i += 1;
            }
            b'[' if d == depth && last_key.as_deref() == Some(key) => {
                // scan elements
                i += 1;
                let mut depth_in = 0usize;
                let mut elem_start = i;
                loop {
                    match b[i] {
                        b'"' => {
                            i += 1;
                            while b[i] != b'"' {
                                if b[i] == b'\\' {
                                    i += 1;
                                }
                                i += 1;
                            }
                            i += 1;
                        }
                        b'{' | b'[' => {
                            depth_in += 1;
                            i += 1;
                        }
                        b'}' => {
                            depth_in -= 1;
                            i += 1;
                        }
                        b']' if depth_in == 0 => {
                            if i > elem_start {
                                out.push((elem_start, i));
                            }
                            return out;
                        }
                        b']' => {
                            depth_in -= 1;
                            i += 1;
                        }
                        b',' if depth_in == 0 => {
                            out.push((elem_start, i));
                            i += 1;
                            elem_start = i;
                        }
                        _ => i += 1,
                    }
                }
            }
            _ => i += 1,
        }
    }
    out
}

fn ack_of(s: &Snapshot) -> BaseAck {
    BaseAck {
        request_id: s.request_id.clone(),
        project_id: s.list.project_id.clone(),
        revision: s.list.revision,
        page_count: s.list.pages.len(),
        list_digest: flashtex_font_engine::sha256::hex(&s.list_digest),
    }
}

fn snapshot_from(request_id: &str, list: &DisplayList, page_bytes: Vec<usize>, text: &str) -> Snapshot {
    let page_digests: Vec<[u8; 32]> = list.pages.iter().map(delta::page_digest).collect();
    let list_digest = delta::list_digest(list, &page_digests);
    Snapshot {
        request_id: request_id.to_string(),
        list: list.clone(),
        page_digests,
        list_digest,
        page_bytes,
        texts: vec![("main.tex".to_string(), text.to_string())],
    }
}

struct Stats {
    deltas: usize,
    fulls: usize,
    delta_bytes: usize,
    full_bytes: usize,
    changed_pages: usize,
    pages: usize,
}

fn gate(sections: usize, edits: usize, min_pages: usize) -> Stats {
    let fonts = FontSet::with_default_dirs(&[]);
    let options = RenderOptions::default();
    let cache = RenderCache::new();
    let base = document(sections);
    let mut installed: Option<Snapshot> = None;
    let mut st = Stats {
        deltas: 0,
        fulls: 0,
        delta_bytes: 0,
        full_bytes: 0,
        changed_pages: 0,
        pages: 0,
    };
    for i in 0..edits {
        let text = edited(&base, i);
        let id = format!("g-{i}");
        // fresh full oracle: no cache, no delta
        let fresh = protocol::handle_line(&request(&id, i + 1, &text, false, None), &fonts, &options, None);
        assert_eq!(fresh.extra_lines.len(), 1, "edit {i}: fresh full sibling");
        let fresh_line = &fresh.extra_lines[0];
        let fresh_pages = array_elements(fresh_line, "pages", 2);
        let fresh_page_bytes: Vec<usize> = fresh_pages.iter().map(|(a, b)| b - a).collect();
        st.pages = fresh_pages.len();
        st.full_bytes += fresh_line.len();
        // delta-mode worker, acknowledging the installed base
        let ack = installed.as_ref().map(ack_of);
        let reply = protocol::handle_line(&request(&id, i + 1, &text, true, ack.as_ref()), &fonts, &options, Some(&cache));
        let result = json::parse(&reply.line).unwrap();
        let echoed: Vec<String> = result
            .get("payload")
            .unwrap()
            .get("layout_capabilities")
            .unwrap()
            .as_arr()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        assert_eq!(reply.extra_lines.len(), 1, "edit {i}: exactly one sibling");
        let sibling = &reply.extra_lines[0];
        let rendered = reply.rendered.as_ref().unwrap();
        if echoed.iter().any(|c| c == delta::CAP_DELTA) {
            assert!(installed.is_some(), "edit {i}: delta without an acknowledged base");
            let env = json::parse(sibling).unwrap();
            assert_eq!(env.get("type").unwrap().as_str(), Some(delta::MESSAGE_TYPE));
            let d = Delta::from_wire(&env).expect("well-formed delta");
            // (1) the changed page objects on the wire are the fresh line's bytes
            let wire_changed = array_elements(sibling, "changed_pages", 2);
            assert_eq!(wire_changed.len(), d.changed.len());
            for (k, &pi) in d.changed.iter().enumerate() {
                let (a, b) = wire_changed[k];
                let (fa, fb) = fresh_pages[pi];
                assert_eq!(&sibling[a..b], &fresh_line[fa..fb], "edit {i}: changed page {} bytes differ from the fresh line", pi + 1);
            }
            // (2) page_bytes equal the fresh line's real page object lengths, all pages
            assert_eq!(d.page_bytes, fresh_page_bytes, "edit {i}: page_bytes vs fresh raw ranges");
            // (3) header parts on the wire equal the fresh line's
            let fp = json::parse(fresh_line).unwrap();
            for k in ["documents", "fonts", "diagnostics", "required_features"] {
                assert_eq!(
                    json::write(env.get("payload").unwrap().get(k).unwrap()),
                    json::write(fp.get("payload").unwrap().get(k).unwrap()),
                    "edit {i}: header part {k}"
                );
            }
            // (4) reconstruct from the installed base; header/changed pages
            //     from the producer's model (the wire bytes of those pages
            //     were proven identical in (1)); byte-identical to fresh
            let changed: Vec<_> = d.changed.iter().map(|&pi| rendered.v2.pages[pi].clone()).collect();
            let header = DisplayList {
                pages: Vec::new(),
                ..rendered.v2.clone()
            };
            let inst = installed.as_ref().unwrap();
            let r = delta::apply(inst, &d, &header, &changed, &id).unwrap_or_else(|e| panic!("edit {i}: refusal {e:?}"));
            let r_line = json::write(&r.to_json(&id));
            assert!(r_line == *fresh_line, "edit {i}: reconstructed line differs ({} vs {} bytes)", r_line.len(), fresh_line.len());
            // (5) exact target size equals the fresh line length
            let feats: Vec<&str> = d.required_features.iter().map(String::as_str).collect();
            assert_eq!(delta::full_line_bytes(&header, &id, &feats, &d.page_bytes), Some(fresh_line.len()), "edit {i}: exact target bytes");
            // (6) the wire delta is smaller than the full line by policy
            assert!(sibling.len() * delta::POLICY_DEN <= fresh_line.len() * delta::POLICY_NUM, "edit {i}: policy");
            st.deltas += 1;
            st.delta_bytes += sibling.len();
            st.changed_pages += d.changed.len();
            installed = Some(snapshot_from(&id, &r, d.page_bytes.clone(), &text));
        } else {
            assert!(sibling == fresh_line, "edit {i}: full sibling differs from fresh");
            st.fulls += 1;
            st.delta_bytes += sibling.len();
            installed = Some(snapshot_from(&id, &rendered.v2, fresh_page_bytes.clone(), &text));
        }
    }
    assert!(st.pages >= min_pages, "{} pages", st.pages);
    eprintln!(
        "{sections} sections, {} pages, {edits} edits: {} deltas / {} full replies; sibling bytes {} vs fresh full {}; changed pages per delta {:.2}",
        st.pages,
        st.deltas,
        st.fulls,
        st.delta_bytes,
        st.full_bytes,
        if st.deltas > 0 { st.changed_pages as f64 / st.deltas as f64 } else { 0.0 }
    );
    st
}

/// Over the wire (real `handle_line`, unchanged 16 MiB line limit). The
/// script's 27-page document cannot be used here: its full `display_list`
/// line is ~24 MB and is declined today (`display_list_declined`), so no
/// base ever exists for it — exactly proposal §8. 20 sections = 14 pages is
/// the largest script size whose full line (11.1 MB) fits.
#[test]
fn delta_wire_reconstruction_is_byte_identical_over_200_edits_of_a_14_page_document() {
    if !lm_available() {
        eprintln!("skipping: Latin Modern not installed");
        return;
    }
    let st = gate(20, 200, 14);
    assert!(st.deltas > st.fulls, "deltas should dominate: {} deltas, {} full", st.deltas, st.fulls);
}

/// The 27-page shape of `tests/incremental.rs`, in-process (no line limit):
/// fresh `render` vs cached `render_cached` + `delta::build` + `delta::apply`,
/// the reconstruction's `to_json` bytes equal to the fresh compile's.
#[test]
fn delta_model_reconstruction_is_byte_identical_over_200_edits_of_a_27_page_document() {
    if !lm_available() {
        eprintln!("skipping: Latin Modern not installed");
        return;
    }
    model_gate(40, 200, 27);
}

fn model_gate(sections: usize, edits: usize, min_pages: usize) {
    use flashtex_compiler::parser::SourceDocument;
    use flashtex_render_pipeline::render_cached;
    let fonts = FontSet::with_default_dirs(&[]);
    let cache = RenderCache::new();
    let base = document(sections);
    let mut installed: Option<Snapshot> = None;
    let (mut deltas, mut fulls, mut changed_total, mut pages) = (0usize, 0usize, 0usize, 0usize);
    for i in 0..edits {
        let text = edited(&base, i);
        let id = format!("m-{i}");
        let docs = [SourceDocument { path: "main.tex", text: &text }];
        let fresh = render_cached(&docs, "main.tex", i as u64 + 1, "gate", &fonts, &RenderOptions::default(), None);
        let fresh_line = json::write(&fresh.v2.to_json(&id));
        let inc = render_cached(&docs, "main.tex", i as u64 + 1, "gate", &fonts, &RenderOptions::default(), Some(&cache));
        pages = fresh.v2.pages.len();
        let texts = vec![("main.tex".to_string(), text.clone())];
        let ack = installed.as_ref().map(ack_of);
        let built = match (&installed, &ack) {
            (Some(inst), Some(a)) => delta::build(inst, a, &inc.v2, &texts).ok(),
            _ => None,
        };
        match built {
            Some(d) => {
                let changed: Vec<_> = d.changed.iter().map(|&pi| inc.v2.pages[pi].clone()).collect();
                let header = DisplayList { pages: Vec::new(), ..inc.v2.clone() };
                // page_bytes: unchanged pages by arithmetic, changed by the writer; all must equal the fresh page objects
                let fresh_pages = array_elements(&fresh_line, "pages", 2);
                assert_eq!(d.page_bytes, fresh_pages.iter().map(|(a, b)| b - a).collect::<Vec<_>>(), "edit {i}: page_bytes");
                let r = delta::apply_with_cap(installed.as_ref().unwrap(), &d, &header, &changed, &id, usize::MAX).unwrap_or_else(|e| panic!("edit {i}: {e:?}"));
                let r_line = json::write(&r.to_json(&id));
                assert!(r_line == fresh_line, "edit {i}: reconstruction differs ({} vs {} bytes)", r_line.len(), fresh_line.len());
                let feats: Vec<&str> = d.required_features.iter().map(String::as_str).collect();
                assert_eq!(delta::full_line_bytes(&header, &id, &feats, &d.page_bytes), Some(fresh_line.len()));
                deltas += 1;
                changed_total += d.changed.len();
                installed = Some(snapshot_from(&id, &r, d.page_bytes.clone(), &text));
            }
            None => {
                let inc_line = json::write(&inc.v2.to_json(&id));
                assert!(inc_line == fresh_line, "edit {i}: cached full differs from fresh");
                fulls += 1;
                let fresh_pages = array_elements(&fresh_line, "pages", 2);
                installed = Some(snapshot_from(&id, &inc.v2, fresh_pages.iter().map(|(a, b)| b - a).collect(), &text));
            }
        }
    }
    assert!(pages >= min_pages, "{pages} pages");
    eprintln!("{sections} sections, {pages} pages, {edits} edits (in-process): {deltas} deltas / {fulls} full; changed pages per delta {:.2}", if deltas > 0 { changed_total as f64 / deltas as f64 } else { 0.0 });
    assert!(deltas > fulls);
}

/// ~2 minutes in release on a loaded machine; run explicitly with `--ignored`.
#[test]
#[ignore = "slow: cargo test --release --test delta -- --ignored"]
fn delta_reconstruction_is_byte_identical_over_30_edits_of_a_107_page_document() {
    if !lm_available() {
        eprintln!("skipping: Latin Modern not installed");
        return;
    }
    gate(160, 30, 107);
}

/// Negotiation: not requested → no delta; requested without an
/// acknowledgement → full; acknowledgement of an older snapshot → full;
/// a `failed`/unrequested reply clears the snapshot.
#[test]
fn delta_negotiation_refusals_answer_full() {
    if !lm_available() {
        eprintln!("skipping: Latin Modern not installed");
        return;
    }
    let fonts = FontSet::with_default_dirs(&[]);
    let options = RenderOptions::default();
    let cache = RenderCache::new();
    let t0 = document(6);
    let t1 = edited(&t0, 0);
    let t2 = edited(&t0, 4);
    let echo = |reply: &protocol::Reply| -> Vec<String> {
        json::parse(&reply.line)
            .unwrap()
            .get("payload")
            .unwrap()
            .get("layout_capabilities")
            .unwrap()
            .as_arr()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect()
    };
    let has_delta = |r: &protocol::Reply| echo(r).iter().any(|c| c == delta::CAP_DELTA);
    // first request: requested, no acknowledgement → full, snapshot retained
    let r0 = protocol::handle_line(&request("a", 1, &t0, true, None), &fonts, &options, Some(&cache));
    assert!(!has_delta(&r0));
    assert!(r0.extra_lines[0].contains("\"type\":\"display_list\""));
    let s0 = cache.delta_snapshot().expect("snapshot after full");
    // acknowledging it → delta
    let r1 = protocol::handle_line(&request("b", 2, &t1, true, Some(&ack_of(&s0))), &fonts, &options, Some(&cache));
    assert!(has_delta(&r1), "echo {:?}", echo(&r1));
    assert!(r1.extra_lines[0].contains("\"type\":\"display_list_delta\""));
    // acknowledging the OLD snapshot again (consumer did not install b) → full, silently
    let r2 = protocol::handle_line(&request("c", 3, &t2, true, Some(&ack_of(&s0))), &fonts, &options, Some(&cache));
    assert!(!has_delta(&r2));
    assert!(r2.extra_lines[0].contains("\"type\":\"display_list\""));
    assert!(!r2.line.contains("display_list_delta"), "no diagnostic for a normal wrong-base full reply");
    // not requested → no delta, and the snapshot is cleared
    let r3 = protocol::handle_line(&request("d", 4, &t2, false, None), &fonts, &options, Some(&cache));
    assert!(!has_delta(&r3));
    assert!(cache.delta_snapshot().is_none(), "snapshot cleared by a reply without a delta request");
    // -delta without display-list-v2 is never accepted
    let line = request("e", 5, &t2, true, None).replace("\"display-list-v2\",", "");
    let r4 = protocol::handle_line(&line, &fonts, &options, Some(&cache));
    assert!(!has_delta(&r4));
    assert!(r4.extra_lines.is_empty());
}

/// Consumer-side refusals on the reference `apply`: forged page_bytes (a
/// changed and an unchanged page, ±1), cap + 1 before any page is built,
/// wrong base, tampered digest.
#[test]
fn delta_apply_refusals() {
    if !lm_available() {
        eprintln!("skipping: Latin Modern not installed");
        return;
    }
    let fonts = FontSet::with_default_dirs(&[]);
    let options = RenderOptions::default();
    let cache = RenderCache::new();
    let t0 = document(6);
    let t1 = edited(&t0, 0);
    let r0 = protocol::handle_line(&request("a", 1, &t0, true, None), &fonts, &options, Some(&cache));
    let s0 = cache.delta_snapshot().unwrap_or_else(|| panic!("no snapshot after r0: {}", &r0.line[..r0.line.len().min(300)]));
    let inst = Snapshot::clone(&s0);
    let fresh_pages = array_elements(&r0.extra_lines[0], "pages", 2);
    assert_eq!(inst.page_bytes, fresh_pages.iter().map(|(a, b)| b - a).collect::<Vec<_>>(), "installed page_bytes are the real writer's");
    let r1 = protocol::handle_line(&request("b", 2, &t1, true, Some(&ack_of(&s0))), &fonts, &options, Some(&cache));
    let env = json::parse(&r1.extra_lines[0]).unwrap();
    let d = Delta::from_wire(&env).unwrap();
    let rendered = r1.rendered.as_ref().unwrap();
    let changed: Vec<_> = d.changed.iter().map(|&pi| rendered.v2.pages[pi].clone()).collect();
    let header = DisplayList {
        pages: Vec::new(),
        ..rendered.v2.clone()
    };
    assert!(!d.changed.is_empty() && d.changed.len() < d.page_count, "the edit must leave both changed and unchanged pages");
    let ok = delta::apply(&inst, &d, &header, &changed, "b").unwrap();
    let feats: Vec<&str> = d.required_features.iter().map(String::as_str).collect();
    let target = delta::full_line_bytes(&header, "b", &feats, &d.page_bytes).unwrap();
    assert_eq!(json::write(&ok.to_json("b")).len(), target);
    // cap + 1: refused before reconstruction, naming both numbers
    assert_eq!(
        delta::apply_with_cap(&inst, &d, &header, &changed, "b", target - 1),
        Err(Refusal::TargetOversize { bytes: target, cap: target - 1 })
    );
    assert!(delta::apply_with_cap(&inst, &d, &header, &changed, "b", target).is_ok());
    // forged page_bytes on a changed page
    let mut forged = Delta { ..clone_delta(&d) };
    forged.page_bytes[d.changed[0]] += 1;
    assert_eq!(delta::apply(&inst, &forged, &header, &changed, "b"), Err(Refusal::PageBytesMismatch(d.changed[0] + 1)));
    // forged page_bytes on an unchanged page
    let unchanged = (0..d.page_count).find(|i| !d.changed.contains(i)).unwrap();
    let mut forged = clone_delta(&d);
    forged.page_bytes[unchanged] -= 1;
    assert_eq!(delta::apply(&inst, &forged, &header, &changed, "b"), Err(Refusal::PageBytesMismatch(unchanged + 1)));
    // wrong base
    let mut wrong = clone_delta(&d);
    wrong.base.list_digest = "0".repeat(64);
    assert_eq!(delta::apply(&inst, &wrong, &header, &changed, "b"), Err(Refusal::BaseMismatch));
    // tampered digest of an unchanged page
    let mut bad = clone_delta(&d);
    bad.page_digests[unchanged][0] ^= 1;
    assert_eq!(delta::apply(&inst, &bad, &header, &changed, "b"), Err(Refusal::DigestMismatch(unchanged + 1)));
}

fn clone_delta(d: &Delta) -> Delta {
    Delta {
        base: d.base.clone(),
        required_features: d.required_features.clone(),
        relocations: d.relocations.clone(),
        page_count: d.page_count,
        page_digests: d.page_digests.clone(),
        page_bytes: d.page_bytes.clone(),
        changed: d.changed.clone(),
        removed_pages: d.removed_pages.clone(),
        list_digest: d.list_digest,
    }
}

/// Relocation arithmetic vs the real writer across digit boundaries, signed
/// moves, escapes and UTF-8 in the page text.
#[test]
fn relocated_page_bytes_match_the_writer() {
    if !lm_available() {
        eprintln!("skipping: Latin Modern not installed");
        return;
    }
    let text = "\\begin{document}\nA \"quoted\" caf\\'e — naïve \\textbf{bold} word $x^2$ here.\n\\end{document}\n";
    let r = render_one(text);
    let page = &r.v2.pages[0];
    let cached = delta::page_bytes(page);
    for (a, d) in [(0usize, 1i64), (0, 3), (0, 83), (0, 984), (0, 99_973), (5, -3), (20, -17)] {
        let rl = delta::Relocation {
            path: "main.tex".into(),
            edit_start: a,
            edit_end: a,
            delta: d,
        };
        let moved = delta::relocate_page(page, std::slice::from_ref(&rl));
        let arith = delta::relocated_page_bytes(page, cached, std::slice::from_ref(&rl));
        match moved {
            Some(m) => assert_eq!(Some(delta::page_bytes(&m)), arith, "delta {d} from {a}"),
            None => assert_eq!(arith, None),
        }
    }
}
