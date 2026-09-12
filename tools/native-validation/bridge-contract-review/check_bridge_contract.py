#!/usr/bin/env python3
"""Linux-safe, pinned-source review signals; never claims native execution.

Known-bad signatures are regression alarms, not a Swift parser or a proof that
other implementations are safe. UTF-8 fixture checks encode the transfer-v1
acceptance oracle; they do not run the native consumer.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess


MAC = "apps/mac/Sources/FlashTeXMac/"


def read_git(repo, sha, path):
    return subprocess.check_output(
        ["git", "-C", str(repo), "show", f"{sha}:{path}"], text=True
    )


def function(source, name):
    """Bound the project's four-space-indented method at its closing line."""
    match = re.search(r"^    (?:private |static |@discardableResult )*func "
                      + re.escape(name) + r"\b.*?(?=^    }\s*$)", source,
                      re.MULTILINE | re.DOTALL)
    return match.group(0) if match else ""


def verify_edit(edit, *, project, path, revision, text):
    """Independent transfer-v1 fixture oracle, using UTF-8 scalar boundaries."""
    if (edit["project_id"], edit["path"], edit["expected_revision"]) != (project, path, revision):
        raise ValueError("identity/revision")
    raw = text.encode("utf-8")
    if hashlib.sha256(raw).hexdigest() != edit["document_before_sha256"]:
        raise ValueError("source hash")
    start, end = edit["start_byte"], edit["end_byte"]
    if type(start) is not int or type(end) is not int or not 0 <= start <= end <= len(raw):
        raise ValueError("range")
    try:
        prefix, removed, suffix = raw[:start].decode(), raw[start:end].decode(), raw[end:].decode()
    except UnicodeDecodeError as error:
        raise ValueError("scalar boundary") from error
    if removed != edit["removed_text"]:
        raise ValueError("removed text")
    return prefix + edit["replacement"] + suffix


def review(sources):
    session = sources["BridgeSession.swift"]
    client = sources["BridgeClient.swift"]
    shell = sources["ShellModel+Bridge.swift"]
    signals = []

    def add(key, status, path, evidence):
        signals.append(dict(check=key, status=status, path=MAC + path, evidence=evidence))

    applied = function(session, "applicationApplied")
    catch = re.search(r"catch\s*\{(.*?)\n        }", applied, re.DOTALL)
    if catch and "sendApplied(" in applied[catch.end():] and not re.search(r"\b(return|throw)\b", catch.group(1)):
        add("receipt_after_ledger_write_failure", "FAIL", "BridgeSession.swift",
            "applicationApplied catches persistence failure, then falls through to sendApplied.")
    load = session[session.find("init(storeDirectory:"):session.find("private static let decoder")]
    if "try? Data(contentsOf: url)" in load:
        add("ledger_read_error_treated_as_empty", "FAIL", "BridgeSession.swift",
            "Data read errors are discarded by try?; unreadable existing ledger can become an empty ledger.")
    reconcile = function(session, "reconcile")
    status_catch = re.search(r"do \{ st = try await status.*?catch \{(.*?)\n            }", reconcile, re.DOTALL)
    if status_catch and ".abandoned" in status_catch.group(1) and "documentBeforeText = nil" in status_catch.group(1):
        add("transient_status_error_destroys_recovery", "FAIL", "BridgeSession.swift",
            "Every capture_status error abandons the entry and erases its pre-edit snapshot, including transport failure.")
    send = function(client, "send")
    if "stdin.fileHandleForWriting.write(contentsOf: line)" in send and not re.search(r"(?:writeQueue|ioQueue)\.async", send):
        add("synchronous_bridge_pipe_write", "FAIL", "BridgeClient.swift",
            "send writes synchronously on the caller; BridgeSession callers run on MainActor. A full pipe during conversion stalls UI.")
    attach = function(shell, "attachBridgeAndWait")
    if "session.onChange =" in attach and "self.bridgeStatus = session.status" in attach and "self.bridge === session" not in attach:
        add("detached_session_updates_current_ui", "FAIL", "ShellModel+Bridge.swift",
            "onChange writes model state without checking active session identity; queued old-session events can overwrite a new session.")
    verify = function(session, "verify")
    required = ["edit.projectId == projectId", "edit.path == path", "edit.expectedRevision == revision",
                "SourceDigest.sha256Hex(text) == edit.documentBeforeSha256", "text.rangeOfUTF8", "== edit.removedText"]
    add("prepared_source_guards_present", "OBSERVED" if all(x in verify for x in required) else "REVIEW",
        "BridgeSession.swift", "Static guard presence only: identity, revision, SHA-256, scalar range and removed text.")
    add("envelope_correlation_present", "OBSERVED" if all(x in client for x in
        ["pending.removeValue(forKey: id)", "header.type != entry.expected", "header.protocolVersion == RuntimeV1.protocolVersion"]) else "REVIEW",
        "BridgeClient.swift", "Static presence of request-ID, reply-type and protocol-version checks; payload identity not proven.")
    positions = [attach.find("session.reconcile("), attach.find("session.open(")]
    add("startup_reconcile_before_open", "OBSERVED" if -1 not in positions and positions == sorted(positions) else "REVIEW",
        "ShellModel+Bridge.swift", "Startup source order only; concurrent edits during awaits require native validation.")
    add("document_transaction_durability", "REVIEW", "BridgeSession.swift",
        "Ledger stores before text and after hash; require native crash test proving source and applied ledger commit durably together before receipt.")
    add("font_and_error_surfaces", "REVIEW", "ShellModel+Bridge.swift",
        "Provider failure text is surfaced. Font substitution, proposal diagnostics, and native visual output need native tests; no Linux visual claim.")
    return signals


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--mac-sha", required=True)
    parser.add_argument("--contract-sha", required=True)
    args = parser.parse_args()
    pinned = {}
    for name, revision in [("mac", args.mac_sha), ("contract", args.contract_sha)]:
        pinned[name] = subprocess.check_output(["git", "-C", str(args.repo), "rev-parse", "--verify", revision + "^{commit}"], text=True).strip()
    contract = read_git(args.repo, pinned["contract"], "docs/contracts/transfer-v1.md")
    sources = {name: read_git(args.repo, pinned["mac"], MAC + name) for name in
               ["BridgeSession.swift", "BridgeClient.swift", "ShellModel+Bridge.swift"]}
    signals = review(sources)
    print(json.dumps({"schema_version": 1, "scope": "Static source signals and separate Python contract oracle; no Swift/Xcode/device execution",
                      "pinned_shas": pinned, "contract_sha256": hashlib.sha256(contract.encode()).hexdigest(),
                      "signals": signals}, indent=2))
    return int(any(s["status"] == "FAIL" for s in signals))


if __name__ == "__main__":
    raise SystemExit(main())
