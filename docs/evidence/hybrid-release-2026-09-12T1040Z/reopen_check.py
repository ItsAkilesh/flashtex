"""Final source / reopen check for typing-bench controller cells.

For each cell dir <work>/controller-<seed>-<ms>ms/ (seed copy + ledger written by the
app during the run), relaunch the helper on that project root + private ledger
root, ask `document`, and compare the durable text with the expected final text:
the seed with typed-200.txt inserted before the last `\\end{document}`.
Usage: reopen_check.py <helper> <typed-200.txt> <bench2 dir> [...]
"""
import hashlib
import json
import os
import subprocess
import sys
import uuid

helper, typed_path = sys.argv[1], sys.argv[2]
typed = open(typed_path, encoding="utf-8").read().replace("\r\n", "\n")


def expected_text(seed_text):
    i = seed_text.rfind("\\end{document}")
    return seed_text + typed if i < 0 else seed_text[:i] + typed + seed_text[i:]


def helper_document(root, ledger, entry):
    cfg = {"session_id": "reopen-" + uuid.uuid4().hex[:8], "project_id": "demo", "entry_path": entry,
           "project_root": root, "private_ledger_root": ledger}
    cfg_path = os.path.join(root, "reopen-config.json")
    json.dump(cfg, open(cfg_path, "w"))
    p = subprocess.Popen([helper, cfg_path], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    frames = [{"protocol_version": 1, "session_id": cfg["session_id"], "id": "d1", "type": "document", "payload": {"path": entry}},
              {"protocol_version": 1, "session_id": cfg["session_id"], "id": "c1", "type": "close", "payload": {}}]
    out, err = p.communicate("".join(json.dumps(f) + "\n" for f in frames), timeout=30)
    for line in out.splitlines():
        f = json.loads(line)
        if f.get("id") == "d1":
            if f["type"] == "error":
                return None, f["payload"]["message"]
            return f["payload"]["document"], None
    return None, "no document reply (stderr: %s)" % err.strip()[-200:]


rows = []
for bench_dir in sys.argv[3:]:
    mode = os.path.basename(bench_dir.rstrip("/"))
    work = open(os.path.join(bench_dir, "work-dir.txt")).read().strip()
    for cell in sorted(os.listdir(work)):
        cell_dir = os.path.join(work, cell)
        if not cell.startswith("controller-") or not os.path.isdir(cell_dir):
            continue
        seed = cell.split("-")[1]
        seed_text = open(os.path.join(work, seed + ".tex"), encoding="utf-8").read()
        expected = expected_text(seed_text)
        # The app names a seeded buffer main.tex inside a session temporary project
        # root (ShellModel.attachController); the log's `launched … for <root> (ledger <root>)`
        # line says which. Reopen exactly that project root + ledger root.
        log = os.path.join(bench_dir, cell + ".log")
        root = ledger = None
        for line in open(log, encoding="utf-8", errors="replace"):
            if "launched " in line and " for " in line and "(ledger " in line:
                root = line.split(" for ", 1)[1].split(" (ledger ")[0].strip()
                ledger = line.split("(ledger ", 1)[1].rsplit(")", 1)[0].strip()
                break
        if not root:
            rows.append((mode, cell, "ERROR", "no launch line in log")); continue
        doc, error = helper_document(root, ledger, "main.tex")
        if doc is None:
            rows.append((mode, cell, "ERROR", error))
            continue
        ok = doc["text"] == expected
        detail = "r%d %s %d bytes" % (doc["revision"], doc["source_sha256"][:12], len(doc["text"].encode()))
        if not ok:
            detail += " (expected %d bytes sha %s)" % (len(expected.encode()), hashlib.sha256(expected.encode()).hexdigest()[:12])
        # The app's final buffer per the log: the last `keystroke:` revision must equal
        # the durable revision's editor revision recorded in `durable:` lines.
        last_key = last_durable = None
        for line in open(log, encoding="utf-8", errors="replace"):
            if "keystroke: revision" in line:
                last_key = int(line.split("keystroke: revision")[1].split()[0])
            if "durable: r" in line:
                part = line.split("durable: r")[1].split()
                last_durable = (int(part[0]), int(part[3]))
        detail += "; last keystroke rev %s, last durable r%s for rev %s" % (last_key, last_durable[0] if last_durable else None, last_durable[1] if last_durable else None)
        rows.append((mode, cell, "OK" if ok and last_durable and last_durable[1] == last_key else "MISMATCH", detail))

print("| mode | cell | reopen | detail |")
print("|---|---|---|---|")
for r in rows:
    print("| %s | %s | %s | %s |" % r)
