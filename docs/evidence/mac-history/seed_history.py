"""Seeds a project + private ledger with durable history so the app's panel has
stacks to show: A -> B -> C -> D, then undo twice (text B; undo 1, redo 2),
then writes B to disk so the app's buffer equals the ledger text on attach."""
import subprocess, json, os, shutil, sys
base = sys.argv[1]
shutil.rmtree(base, ignore_errors=True)
proj, ledger = os.path.join(base, "project"), os.path.join(base, "ledger")
os.makedirs(proj); os.makedirs(ledger)
texts = [
    "\\documentclass{article}\n\\begin{document}\nDurable history demo.\n\\end{document}\n",
    "\\documentclass{article}\n\\begin{document}\nDurable history demo, first edit.\n\\end{document}\n",
    "\\documentclass{article}\n\\begin{document}\nDurable history demo, first edit, second edit.\n\\end{document}\n",
    "\\documentclass{article}\n\\begin{document}\nDurable history demo, first edit, second edit, third.\n\\end{document}\n",
]
with open(os.path.join(proj, "main.tex"), "w") as f: f.write(texts[0])
cfg = os.path.join(base, "config.json")
json.dump({"session_id":"seed","project_id":"demo","entry_path":"main.tex","project_root":proj,"private_ledger_root":ledger}, open(cfg, "w"))
p = subprocess.Popen([os.environ["FLASHTEX_PREVIEW_CONTROLLER"], cfg], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
def call(id, type, payload):
    p.stdin.write(json.dumps({"protocol_version":1,"session_id":"seed","id":id,"type":type,"payload":payload})+"\n"); p.stdin.flush()
    while True:
        r = json.loads(p.stdout.readline())
        if r["type"] in ("result", "error") and r["id"] == id: return r
assert json.loads(p.stdout.readline())["type"] == "ready"
doc = call("d", "document", {"path": "main.tex"})["payload"]["document"]
rev, sha = doc["revision"], doc["source_sha256"]
for i, t in enumerate(texts[1:], 1):
    d = call(f"e{i}", "edit", {"path":"main.tex","expected_revision":rev,"expected_sha256":sha,"text":t})["payload"]["document"]
    rev, sha = d["revision"], d["source_sha256"]
for i in range(2):
    r = call(f"u{i}", "undo", {"path":"main.tex","command":{"command_id":f"seed-undo-{i}","expected_revision":rev,"expected_sha256":sha}})
    d = r["payload"]["history"]["document"]; rev, sha = d["revision"], d["source_sha256"]
assert d["text"] == texts[1], d["text"]
print("status", call("h", "history_status", {"path":"main.tex"})["payload"])
call("c", "close", {}); p.stdin.close(); p.wait()
with open(os.path.join(proj, "main.tex"), "w") as f: f.write(texts[1])
print("seeded", base, "durable r%d" % rev)
