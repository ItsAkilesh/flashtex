//! Independent structural checks on a finished PDF byte string.
//!
//! These read the file back the way a viewer's first pass would: header,
//! `startxref`, the cross-reference table, and each `N 0 obj` header at the
//! offset the table claims. They exist so the writer's tests catch the classic
//! wrong-offset bug rather than trusting the writer's own bookkeeping. They are
//! also handy for the CLI's `--verify` flag.

/// What the cross-reference pass found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Structure {
    /// Number of in-use objects (the free object 0 is excluded).
    pub object_count: usize,
    pub xref_offset: usize,
    /// Object numbers of every stream object, in file order.
    pub stream_objects: Vec<usize>,
}

/// One `Tj` as written by this crate's writer, recovered from a content stream.
#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    pub font_size: f64,
    pub x: f64,
    pub y: f64,
    /// Raw string bytes after undoing the writer's escapes.
    pub bytes: Vec<u8>,
}

/// Checks header, trailer, and that every xref entry points at its object.
pub fn check_structure(pdf: &[u8]) -> Result<Structure, String> {
    if !pdf.starts_with(b"%PDF-1.4\n") {
        return Err("missing %PDF-1.4 header".into());
    }
    let tail = &pdf[pdf.len().saturating_sub(64)..];
    let eof = find(tail, b"%%EOF").ok_or("missing %%EOF trailer marker")?;
    if tail[eof + 5..].iter().any(|b| !b.is_ascii_whitespace()) {
        return Err("bytes after %%EOF".into());
    }
    let sx = rfind(pdf, b"startxref\n").ok_or("missing startxref")?;
    let xref_offset: usize = ascii_line(pdf, sx + "startxref\n".len())?
        .parse()
        .map_err(|_| "startxref is not a number")?;
    if !pdf[xref_offset..].starts_with(b"xref\n") {
        return Err(format!("startxref {xref_offset} does not point at 'xref'"));
    }
    let mut pos = xref_offset + "xref\n".len();
    let header = ascii_line(pdf, pos)?;
    pos += header.len() + 1;
    let (first, count) = header
        .split_once(' ')
        .and_then(|(a, b)| Some((a.parse::<usize>().ok()?, b.parse::<usize>().ok()?)))
        .ok_or("malformed xref subsection header")?;
    if first != 0 {
        return Err("xref must start at object 0".into());
    }
    let mut stream_objects = Vec::new();
    for n in 0..count {
        let entry = pdf
            .get(pos..pos + 20)
            .ok_or_else(|| format!("xref entry {n} truncated"))?;
        pos += 20;
        let entry = std::str::from_utf8(entry).map_err(|_| format!("xref entry {n} not ASCII"))?;
        if entry.len() != 20 || !entry.ends_with(" \n") {
            return Err(format!("xref entry {n} is not 20 bytes: {entry:?}"));
        }
        let offset: usize = entry[0..10]
            .parse()
            .map_err(|_| format!("xref entry {n} offset"))?;
        let kind = &entry[17..18];
        match (n, kind) {
            (0, "f") => continue,
            (0, _) => return Err("object 0 must be free".into()),
            (_, "n") => {}
            (_, k) => return Err(format!("xref entry {n} has unexpected type {k:?}")),
        }
        let expect = format!("{n} 0 obj\n");
        if !pdf
            .get(offset..)
            .is_some_and(|s| s.starts_with(expect.as_bytes()))
        {
            return Err(format!(
                "xref entry {n} offset {offset} does not point at {expect:?}"
            ));
        }
        let body_start = offset + expect.len();
        let end = find(&pdf[body_start..], b"endobj").ok_or(format!("object {n} has no endobj"))?;
        if find(&pdf[body_start..body_start + end], b"stream\n").is_some() {
            stream_objects.push(n);
        }
    }
    if !pdf[pos..].starts_with(b"trailer\n") {
        return Err("xref table not followed by trailer".into());
    }
    let trailer = ascii_line(pdf, pos + "trailer\n".len())?;
    if !trailer.contains(&format!("/Size {count}")) {
        return Err(format!(
            "trailer /Size disagrees with xref count {count}: {trailer}"
        ));
    }
    if !trailer.contains("/Root 1 0 R") {
        return Err(format!("trailer has no /Root 1 0 R: {trailer}"));
    }
    Ok(Structure {
        object_count: count - 1,
        xref_offset,
        stream_objects,
    })
}

/// Returns the raw bytes of the given stream object's data.
pub fn stream_data(pdf: &[u8], object: usize) -> Result<Vec<u8>, String> {
    let head = format!("\n{object} 0 obj\n");
    let at = find(pdf, head.as_bytes()).ok_or(format!("object {object} not found"))? + 1;
    let dict_start = at + head.len() - 1;
    let dict = ascii_line(pdf, dict_start)?;
    let len_at = dict
        .find("/Length ")
        .ok_or(format!("object {object} has no /Length"))?;
    let length: usize = dict[len_at + 8..]
        .split(|c: char| !c.is_ascii_digit())
        .next()
        .and_then(|d| d.parse().ok())
        .ok_or(format!("object {object} has non-numeric /Length"))?;
    let data_start = dict_start + dict.len() + 1 + "stream\n".len();
    let data = pdf
        .get(data_start..data_start + length)
        .ok_or(format!("object {object} stream truncated"))?;
    if !pdf[data_start + length..].starts_with(b"\nendstream") {
        return Err(format!(
            "object {object} /Length {length} does not end at endstream"
        ));
    }
    Ok(data.to_vec())
}

/// Recovers each `Tj` from a content stream written by [`crate::writer`].
pub fn placements(content: &[u8]) -> Result<Vec<Placement>, String> {
    let mut out = Vec::new();
    let mut font_size = None;
    let mut td = None;
    let mut pos = 0;
    while pos < content.len() {
        let end = content[pos..]
            .iter()
            .position(|&b| b == b'\n')
            .map_or(content.len(), |e| pos + e);
        let line = &content[pos..end];
        pos = end + 1;
        if line.ends_with(b" Tf") {
            let s = std::str::from_utf8(line).map_err(|_| "Tf line not ASCII")?;
            let mut parts = s.split(' ');
            let font = parts.next().unwrap_or("");
            if font != "/F1" {
                return Err(format!("unexpected font resource {font}"));
            }
            font_size = Some(
                parts
                    .next()
                    .unwrap_or("")
                    .parse::<f64>()
                    .map_err(|_| "Tf size")?,
            );
        } else if line.ends_with(b" Td") {
            let s = std::str::from_utf8(line).map_err(|_| "Td line not ASCII")?;
            let mut parts = s.split(' ');
            let x = parts
                .next()
                .unwrap_or("")
                .parse::<f64>()
                .map_err(|_| "Td x")?;
            let y = parts
                .next()
                .unwrap_or("")
                .parse::<f64>()
                .map_err(|_| "Td y")?;
            td = Some((x, y));
        } else if line.starts_with(b"(") && line.ends_with(b") Tj") {
            let (x, y) = td.take().ok_or("Tj without preceding Td")?;
            let font_size = font_size.ok_or("Tj without Tf")?;
            out.push(Placement {
                font_size,
                x,
                y,
                bytes: unescape(&line[1..line.len() - 4]),
            });
        }
    }
    Ok(out)
}

fn unescape(s: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if s[i] == b'\\' && i + 1 < s.len() {
            out.push(match s[i + 1] {
                b'n' => b'\n',
                b'r' => b'\r',
                other => other,
            });
            i += 2;
        } else {
            out.push(s[i]);
            i += 1;
        }
    }
    out
}

fn ascii_line(pdf: &[u8], start: usize) -> Result<String, String> {
    let rest = pdf.get(start..).ok_or("offset past end of file")?;
    let end = rest.iter().position(|&b| b == b'\n').unwrap_or(rest.len());
    String::from_utf8(rest[..end].to_vec()).map_err(|_| format!("non-ASCII line at {start}"))
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

fn rfind(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).rposition(|w| w == needle)
}
