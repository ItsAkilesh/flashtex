# Text and adopted-delimiter interaction acceptance

Twelve requests exercised on exactbc737126 + Text-only8b676e72 patch, binary
hash in provenance.json (same tested combination as5b250b22). Every persistent
reply equals a separate fresh-process reply as complete JSON. This is an incremental
correctness/recovery gate, not TeX reference parity or native paint timing.

Cases cover paired delimiters, missing-right then repair, explicit unknown delimiter,
comment joining before/inside Text, fraction/script nesting, unsupported Text command
then repair, escaped literal braces, null delimiter, macro argument and Unicode prefix.
All exposed item/diagnostic spans were checked against exact main.tex UTF8 byte bounds
and boundaries. Paired/repair/macro/null cases retained the 'and' item, comment joining
retained 'ab', and malformed delimiters retained their explicit diagnostics.
The unnegotiated fraction-rule export warning in the scripts case remains visible;
we did not suppress it to make the fixture clean.

request.jsonl and fresh-response.jsonl are the portable replay inputs. The latter was
captured with one direct compiler process per request (10second process timeout).
persistent-response.jsonl and timing.json come from the bounded protocol probe,
which enforces per-request identity, complete-frame equality, final EOF and exit.

From repository root after rebuilding the exact candidate:

```sh
python3 crates/preview-controller/tools/bounded_protocol_probe.py \
  --binary /absolute/path/to/flashtex-compiler \
  --requests crates/preview-controller/docs/handoffs/text-delimiter-session/request.jsonl \
  --expected crates/preview-controller/docs/handoffs/text-delimiter-session/fresh-response.jsonl \
  --output /absolute/path/to/new-replay-directory
```

Keep output path new. Review intentional diagnostic changes on a newer compiler;
do not overwrite expected output solely to obtain a pass. No authoritative compiler,
producer or native source was edited. All capture processes exited successfully.
