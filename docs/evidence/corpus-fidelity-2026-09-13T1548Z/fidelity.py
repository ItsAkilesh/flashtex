#!/usr/bin/env python3
"""Corpus fidelity driver (lane mac-corpus-fidelity). Runs every fixture through
flashtex-render (worker mode, all project documents sent), compares the v2 word
geometry against the pdflatex reference PDF (tools/visual-oracle/pdftext.py) and
ranks pages by defect severity. Standard library only."""
import argparse, json, os, re, subprocess, sys, time, statistics

REPO = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")) if False else None

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", required=True)
    ap.add_argument("--render", required=True)
    ap.add_argument("--ext", required=True, help="extended-tex-corpus root")
    ap.add_argument("--out", required=True)
    ap.add_argument("--only", action="append", default=[])
    ap.add_argument("--no-pdf", action="store_true")
    a = ap.parse_args()
    sys.path.insert(0, os.path.join(a.repo, "tools", "visual-oracle"))
    sys.path.insert(0, os.path.join(a.repo, "tools", "real-world-corpus"))
    import pdftext, rank
    os.makedirs(a.out, exist_ok=True)
    fixtures = []
    rw = os.path.join(a.repo, "fixtures", "real-world")
    for fid in sorted(os.listdir(rw)):
        d = os.path.join(rw, fid)
        if not os.path.isdir(d):
            continue
        texs = [f for f in os.listdir(d) if f.endswith(".tex")]
        entry = "main.tex" if "main.tex" in texs else texs[0]
        ref = None
        for cand in ["reference.pdf"] + [f for f in os.listdir(d) if f.endswith("-reference.pdf")]:
            if os.path.exists(os.path.join(d, cand)):
                ref = os.path.join(d, cand); break
        files = []
        for root, _, fs in os.walk(d):
            for f in fs:
                if f.endswith((".tex", ".sty", ".cls", ".bib")):
                    files.append(os.path.relpath(os.path.join(root, f), d))
        fixtures.append(dict(id="rw/" + fid, dir=d, entry=entry, files=sorted(files), ref=ref, expect="success", engine="pdflatex"))
    man = json.load(open(os.path.join(a.ext, "manifest.json")))
    for c in man["cases"]:
        d = os.path.join(a.ext, "cases", c["id"])
        ref = os.path.join(a.ext, "references", c["id"], "main.pdf")
        fixtures.append(dict(id="ext/" + c["id"], dir=d, entry=c["entry"], files=[f for f in c["files"] if f.endswith((".tex", ".sty", ".cls", ".bib"))],
                             ref=ref if os.path.exists(ref) else None, expect=c["expect"], engine=c["engine"]))
    if a.only:
        fixtures = [f for f in fixtures if any(o in f["id"] for o in a.only)]
    results = []
    for fx in fixtures:
        r = run_one(a, fx, pdftext, rank)
        results.append(r)
        print(f"{fx['id']:40} {r['summary']}", flush=True)
    json.dump(results, open(os.path.join(a.out, "results.json"), "w"), indent=1)
    write_table(a, results)


def sev_rank(r):
    # lower = worse
    if r["expect"] == "error":
        return (9, 0)  # reported separately
    if r["status"] in ("no_reply", "crash"):
        return (0, 0)
    if r["ref_pages"] is not None and r["pages"] != r["ref_pages"]:
        return (1, -abs(r["pages"] - r["ref_pages"]))
    if r["errors"]:
        return (2, -r["errors"])
    if r["overfull"]:
        return (3, -r["overfull"])
    md = r.get("max_delta") or 0.0
    if md > 2.0:
        return (4, -md)
    if r["underfull"]:
        return (5, -r["underfull"])
    return (6, -md)


def run_one(a, fx, pdftext, rank):
    docs = []
    for f in fx["files"]:
        p = os.path.join(fx["dir"], f)
        try:
            text = open(p, encoding="utf-8").read()
        except UnicodeDecodeError:
            text = open(p, encoding="latin-1").read()
        docs.append({"path": f, "text": text})
    req = {"protocol_version": 1, "id": fx["id"], "type": "compile",
           "payload": {"project_id": fx["id"].replace("/", "-"), "revision": 1, "entry_path": fx["entry"], "documents": docs,
                       "layout_capabilities": ["display-list-v2", "rules-v1", "font-hints-v1"]}}
    slug = fx["id"].replace("/", "__")
    v2 = os.path.join(a.out, slug + ".v2.json")
    pdf = os.path.join(a.out, slug + ".pdf")
    cmd = [a.render, "--v2", v2, "--font-dir", os.path.join(a.repo, "apps", "mac", "Fonts")]
    if not a.no_pdf:
        cmd += ["--pdf", pdf]
    t0 = time.time()
    try:
        pr = subprocess.run(cmd, input=json.dumps(req) + "\n", capture_output=True, text=True, timeout=180, cwd=a.repo)
    except subprocess.TimeoutExpired:
        return dict(id=fx["id"], expect=fx["expect"], engine=fx["engine"], status="no_reply", pages=0, ref_pages=None, errors=0, overfull=0, underfull=0, summary="TIMEOUT", diags=[])
    wall = time.time() - t0
    open(os.path.join(a.out, slug + ".stderr"), "w").write(pr.stderr)
    line = pr.stdout.splitlines()[0] if pr.stdout.strip() else None
    r = dict(id=fx["id"], expect=fx["expect"], engine=fx["engine"], wall=round(wall, 2), pages=0, ref_pages=None, errors=0, overfull=0, underfull=0, diags=[], max_delta=None, geometry=[])
    if not line:
        r["status"] = "crash" if pr.returncode != 0 else "no_reply"
        panic = [l for l in pr.stderr.splitlines() if "panicked" in l]
        r["summary"] = f"{r['status']} exit={pr.returncode} {panic[:1]}"
        return r
    env = json.loads(line)
    if env.get("type") == "error":
        r["status"] = "error_envelope"
        r["summary"] = "error envelope: " + json.dumps(env.get("payload"))[:200]
        r["diags"] = [{"severity": "error", "code": env["payload"].get("code"), "message": env["payload"].get("message")}]
        r["errors"] = 1
        return r
    pl = env["payload"]
    r["status"] = pl.get("status")
    r["pages"] = len(pl.get("pages") or [])
    diags = pl.get("diagnostics") or []
    r["diags"] = [{"severity": d.get("severity"), "code": d.get("code"), "message": d.get("message"), "source": d.get("source")} for d in diags]
    r["errors"] = sum(1 for d in diags if d.get("severity") == "error")
    r["overfull"] = sum(1 for d in diags if (d.get("code") or "").startswith("overfull"))
    r["underfull"] = sum(1 for d in diags if (d.get("code") or "").startswith("underfull"))
    if fx["ref"] and os.path.exists(fx["ref"]):
        try:
            refpages = pdftext.page_words(fx["ref"])
        except Exception as e:
            refpages = None
            r["ref_error"] = str(e)
        if refpages is not None:
            r["ref_pages"] = len(refpages)
            if os.path.exists(v2):
                dl = json.load(open(v2))
                cand = rank.v2_words(dl)
                for i, rp in enumerate(refpages):
                    if i >= len(cand):
                        break
                    g = rank.geometry_page(rp["words"], cand[i]["words"], diags, 6)
                    g["page"] = i + 1
                    r["geometry"].append(g)
                deltas = [g["max_delta"] for g in r["geometry"] if g.get("max_delta") is not None]
                r["max_delta"] = max(deltas) if deltas else None
                r["unaligned_ref"] = sum(g["reference_unaligned"] for g in r["geometry"])
                r["aligned"] = sum(g["aligned"] for g in r["geometry"])
                r["ref_words"] = sum(g["reference_words"] for g in r["geometry"])
    r["summary"] = (f"{r['status']} pages={r['pages']}/{r['ref_pages']} err={r['errors']} over={r['overfull']} under={r['underfull']} "
                    f"maxΔ={r['max_delta']} aligned={r.get('aligned')}/{r.get('ref_words')} {wall:.1f}s")
    return r


def write_table(a, results):
    rows = sorted(results, key=sev_rank)
    out = [f"| rank | fixture | status | pages (ours/ref) | errors | overfull | underfull | max word Δ (bp) | aligned/ref words | worst page | top defect |", "|---|---|---|---|---|---|---|---|---|---|---|"]
    for i, r in enumerate(rows, 1):
        worst = None
        if r.get("geometry"):
            worst = max(r["geometry"], key=lambda g: g.get("max_delta") or 0)
        top = ""
        if worst and worst.get("top"):
            t = worst["top"][0]
            top = f"`{t['text']}` dx={t['dx']} dy={t['dy']} ({t['owner'].split(' — ')[-1][:60]})"
        if r["errors"]:
            errs = [d for d in r["diags"] if d["severity"] == "error"]
            top = f"error[{errs[0]['code']}] {errs[0]['message'][:90]}"
        elif r["overfull"] and not top:
            ov = [d for d in r["diags"] if (d["code"] or "").startswith("overfull")]
            top = f"{ov[0]['code']}: {ov[0]['message'][:80]}"
        md = r.get("max_delta")
        out.append(f"| {i} | {r['id']} | {r['status']} | {r['pages']}/{r['ref_pages']} | {r['errors']} | {r['overfull']} | {r['underfull']} | "
                   f"{md if md is None else round(md, 2)} | {r.get('aligned')}/{r.get('ref_words')} | {worst['page'] if worst else ''} | {top} |")
    open(os.path.join(a.out, "table.md"), "w").write("\n".join(out) + "\n")
    print("\n".join(out))


if __name__ == "__main__":
    main()
