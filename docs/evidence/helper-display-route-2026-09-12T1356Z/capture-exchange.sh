#!/usr/bin/env bash
# Captures the EXACT helper wire (both directions) for one display-candidate
# exchange through the real FlashTeXMac: configure -> edit -> v1 preview ->
# update{kind:"display_candidate"} -> native paint decision (app log).
# The helper runs behind `tee`; nothing is redacted or reformatted. Absolute
# paths of this machine appear in the frames (project root, ledger) and in
# the app log; they are listed in exchange/README.md.
# Usage: FLASHTEX_RENDER=<flashtex-render> [FLASHTEX_PREVIEW_CONTROLLER=<helper>] capture-exchange.sh
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
MAC="$ROOT/apps/mac"
RENDER="${FLASHTEX_RENDER:?set FLASHTEX_RENDER}"
HELPER="${FLASHTEX_PREVIEW_CONTROLLER:-$ROOT/crates/preview-controller/target/release/flashtex-preview-controller}"
APP="$MAC/.build/release/FlashTeXMac"
[[ -x "$RENDER" && -x "$HELPER" && -x "$APP" ]] || { echo "need built producer/helper/app (run.sh builds the app)" >&2; exit 1; }
OUT="$HERE/exchange"; rm -rf "$OUT"; mkdir -p "$OUT"
WORK="$MAC/build/helper-display-route/exchange-$(date -u +%Y-%m-%dT%H%M%SZ)"; mkdir -p "$WORK"
cat > "$WORK/main.tex" <<'TEX'
\documentclass{article}
\begin{document}
Helper display candidates: one exact exchange.
\end{document}
TEX
printf 'ab!' > "$WORK/script.txt"   # three keystrokes; the bench stops after the last paints
cat > "$WORK/helper-tee.sh" <<SH
#!/bin/sh
exec 2>>"$OUT/helper-stderr.log"
tee -a "$OUT/helper-stdin.jsonl" | "$HELPER" "\$@" | tee -a "$OUT/helper-stdout.jsonl"
SH
chmod +x "$WORK/helper-tee.sh"
echo "==> $(uptime)"
(
  cd "$MAC"
  env FLASHTEX_REPO="$ROOT" FLASHTEX_AUTOATTACH=1 FLASHTEX_NO_ACTIVATE=1 \
    FLASHTEX_LM_DIR="$MAC/Fonts" FLASHTEX_FONT_DIRS="$MAC/Fonts" \
    FLASHTEX_PREVIEW_CONTROLLER="$WORK/helper-tee.sh" FLASHTEX_COMPILER="$RENDER" FLASHTEX_CONTROLLER_LEDGER_ROOT="$WORK/ledger" \
    FLASHTEX_PREVIEW_V2=1 FLASHTEX_DISPLAY_CANDIDATES=1 \
    FLASHTEX_SEED_FILE="$WORK/main.tex" FLASHTEX_LOG="$OUT/app.log" \
    FLASHTEX_TYPING_BENCH="$WORK/script.txt" FLASHTEX_TYPING_BENCH_MS=300 FLASHTEX_TYPING_BENCH_OUT="$OUT/bench.json" \
    FLASHTEX_TYPING_BENCH_SETTLE_MS=20000 FLASHTEX_TYPING_BENCH_MAX_MS=30000 \
    "$APP" >/dev/null 2>&1 &
  pid=$!
  for _ in $(seq 1 120); do kill -0 "$pid" 2>/dev/null || break; sleep 0.5; done
  if kill -0 "$pid" 2>/dev/null; then echo "timed out; killing pid $pid" >&2; kill "$pid" 2>/dev/null || true; fi
  wait "$pid" 2>/dev/null || true
)
python3 - "$OUT" "$ROOT" "$HELPER" "$RENDER" <<'PY'
import hashlib, json, os, re, sys
out, root, helper, render = sys.argv[1:5]
sha = lambda p: hashlib.sha256(open(p, "rb").read()).hexdigest()[:16]
def frames(path):
    rows = []
    for n, line in enumerate(open(path, encoding="utf-8", errors="replace"), 1):
        try:
            f = json.loads(line)
        except Exception as e:  # noqa: BLE001
            rows.append((n, len(line), "UNPARSEABLE", "", "", str(e))); continue
        p = f.get("payload") or {}
        kind = p.get("kind") if isinstance(p, dict) else ""
        extra = ""
        if f.get("type") == "update" and kind == "display_candidate":
            extra = "request_id=%s compile_revision=%s source_versions=%s membership_generation=%s display_list=%d bytes untrusted=%s source_actions_enabled=%s" % (
                p.get("request_id"), p.get("compile_revision"), p.get("source_versions"), p.get("membership_generation"),
                len(json.dumps(p.get("display_list"), separators=(",", ":"))), p.get("untrusted"), p.get("source_actions_enabled"))
        elif f.get("type") == "update" and kind == "preview":
            r = p.get("result") or {}
            extra = "request_id=%s compile_revision=%s source_versions=%s accepted=%s" % (p.get("request_id"), p.get("compile_revision"), p.get("source_versions"), (r.get("payload") or {}).get("layout_capabilities"))
        elif f.get("type") in ("configure_display_candidates", "configure_layout", "edit", "ready", "error"):
            extra = json.dumps({k: v for k, v in p.items() if k not in ("text", "documents", "ops")}, separators=(",", ":"))[:300]
        elif f.get("type") == "result":
            extra = ",".join(sorted(p.keys())) if isinstance(p, dict) else ""
        rows.append((n, len(line), f.get("type"), f.get("id"), kind or "", extra))
    return rows
lines = ["# One exact display-candidate exchange (helper wire, unredacted)\n",
         "Helper `%s` (sha256 %s…); producer `%s` (sha256 %s…); app `%s` @ `%s`.\n" % (helper, sha(helper), render, sha(render),
            os.popen("git -C '%s' rev-parse --abbrev-ref HEAD" % root).read().strip(), os.popen("git -C '%s' rev-parse --short HEAD" % root).read().strip()),
         "`helper-stdin.jsonl` = every frame the shell wrote to the helper; `helper-stdout.jsonl` = every frame the helper wrote back, byte for byte "
         "(the `display_list` sibling is inside the `update{kind:\"display_candidate\"}` line); `app.log` = the shell's log including the native gate's "
         "admission / validation / paint decisions; `bench.json` = the 3-keystroke bench summary. Absolute paths of this machine present in the frames: "
         "the project root and ledger under `%s/apps/mac/build/helper-display-route/` and `/var/folders/...` temp dirs; nothing was removed.\n" % root]
for name in ("helper-stdin.jsonl", "helper-stdout.jsonl"):
    lines.append("\n## %s\n\n| line | bytes | type | id | kind | detail |\n|---:|---:|---|---|---|---|" % name)
    for n, b, t, i, k, e in frames(os.path.join(out, name)):
        lines.append("| %d | %d | %s | %s | %s | %s |" % (n, b, t, i, k, str(e).replace("|", "\\|")))
lines.append("\n## app.log (native decisions)\n\n```")
for line in open(os.path.join(out, "app.log"), encoding="utf-8", errors="replace"):
    if re.search(r"display-candidate|controller ready|configure|status: revision|declined|violation", line): lines.append(line.rstrip())
lines.append("```")
open(os.path.join(out, "README.md"), "w", encoding="utf-8").write("\n".join(lines) + "\n")
print(open(os.path.join(out, "README.md"), encoding="utf-8").read())
PY
