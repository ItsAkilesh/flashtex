#!/usr/bin/env python3
"""Validate a pinned FlashTeX companion revision without changing its checkout.

The tool exports Git revisions into temporary directories.  It can therefore
compare a known-bad FT-004 revision with a repair candidate while keeping the
worker's repository and `apps/companion` untouched.
"""

from __future__ import annotations

import argparse
import base64
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
from typing import Any


PROJECT = Path("apps/companion/FlashTeXCompanion.xcodeproj/project.pbxproj")
PAYLOAD = Path("apps/companion/FlashTeXCompanion/Models/CapturePayload.swift")
VALIDATOR = Path("apps/companion/FlashTeXCompanion/Services/ImageValidator.swift")
STORE = Path("apps/companion/FlashTeXCompanion/Models/CaptureStore.swift")
TRANSPORT = Path("apps/companion/FlashTeXCompanion/Services/CaptureTransport.swift")
BONJOUR_TRANSPORT = Path("apps/companion/FlashTeXCompanion/Services/BonjourTransport.swift")
INFO_PLIST = Path("apps/companion/FlashTeXCompanion/Info.plist")
MAC_NEARBY_LISTENER = Path("apps/mac/Sources/FlashTeXMac/NearbyListener.swift")
CAPTURE_FIXTURE = Path("protocol/fixtures/capture-submission.json")
PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"
JPEG_SIGNATURE = b"\xff\xd8\xff"


def run(command: list[str], cwd: Path | None = None, timeout: int = 120, output_limit: int = 4000) -> dict[str, Any]:
    """Run a command and retain concise, JSON-safe evidence."""
    try:
        completed = subprocess.run(
            command,
            cwd=cwd,
            text=True,
            capture_output=True,
            timeout=timeout,
            check=False,
        )
        output = (completed.stdout + completed.stderr).strip()
        return {
            "command": command,
            "exit_code": completed.returncode,
            "output": output[-output_limit:],
        }
    except FileNotFoundError:
        return {"command": command, "exit_code": None, "output": "command not found"}
    except subprocess.TimeoutExpired:
        return {"command": command, "exit_code": None, "output": f"timed out after {timeout}s"}


def export_revision(repo: Path, revision: str, destination: Path) -> None:
    """Create a disposable source tree from a Git revision."""
    archive = subprocess.run(
        ["git", "-C", str(repo), "archive", "--format=tar", revision],
        capture_output=True,
        check=True,
    ).stdout
    archive_path = destination / "source.tar"
    archive_path.write_bytes(archive)
    with tarfile.open(archive_path) as tar:
        # `git archive` is local input, but do not follow a revision's symlinks
        # or absolute/path-traversal members outside the disposable export.
        if hasattr(tarfile, "data_filter"):
            tar.extractall(destination / "source", filter="data")
        else:
            for member in tar.getmembers():
                target = (destination / "source" / member.name).resolve()
                if not target.is_relative_to((destination / "source").resolve()) or member.issym() or member.islnk():
                    raise ValueError(f"unsafe Git archive member: {member.name}")
            tar.extractall(destination / "source")


def resolve_revision(repo: Path, revision: str) -> str:
    return subprocess.run(
        ["git", "-C", str(repo), "rev-parse", f"{revision}^{{commit}}"],
        text=True,
        capture_output=True,
        check=True,
    ).stdout.strip()


def pbx_findings(project_text: str) -> list[str]:
    """Detect object definitions incorrectly spliced into PBX reference lists."""
    findings: list[str] = []
    for block_name, body in re.findall(
        r"(children|files|buildPhases|targets)\s*=\s*\((.*?)\);", project_text, re.DOTALL
    ):
        if re.search(r"/\*.*?\*/\s*=\s*\{", body, re.DOTALL):
            findings.append(f"inline PBX object definition inside {block_name} list")
    if re.search(r"productRefGroup\s*=\s*[^;]*=\s*\{", project_text, re.DOTALL):
        findings.append("inline PBX object definition in productRefGroup")
    return findings


def target_findings(project_text: str) -> list[str]:
    targets = re.findall(r"isa\s*=\s*PBXNativeTarget;.*?name\s*=\s*([^;]+);", project_text, re.DOTALL)
    normalized = {target.strip().strip('"') for target in targets}
    if not any("Test" in target for target in normalized):
        return ["no XCTest PBXNativeTarget found"]
    return []


def detected_mime(data: bytes) -> str | None:
    if data.startswith(PNG_SIGNATURE):
        return "image/png"
    if data.startswith(JPEG_SIGNATURE):
        return "image/jpeg"
    return None


def source_mime_findings(payload_source: str, validator_source: str, store_source: str = "") -> list[str]:
    findings: list[str] = []
    payload_png = "pngData()" in payload_source and 'mimeType: "image/png"' in payload_source
    validator_can_jpeg = "jpegData(" in validator_source
    store_preserves_validated_encoding = (
        "imageData: encodedData" in store_source and "mimeType: mimeType" in store_source
    )
    if validator_can_jpeg and payload_png and not store_preserves_validated_encoding:
        findings.append("validator may produce JPEG but envelope always serializes PNG with image/png")
    if "acceptedMIMETypes" in validator_source and "image/jpeg" in validator_source and not validator_can_jpeg:
        findings.append("JPEG declared accepted but no JPEG encoding path found")
    return findings


def delivery_findings(store_source: str, transport_source: str) -> list[str]:
    """Detect whether an interrupted submission can be cancelled or retried.

    These are intentionally conservative source-level gates.  They do not claim
    a network test occurred; instead they make missing recovery semantics visible
    until a simulator/device test can exercise the concrete transport.
    """
    findings: list[str] = []
    combined = store_source + "\n" + transport_source
    if not re.search(r"\b(cancel|cancelCapture|cancelSubmission)\b", combined):
        findings.append("no cancellation API found for an in-flight capture")
    if not re.search(r"\b(retry|retryCapture|retrySubmission)\b", combined):
        findings.append("no retry API found for a failed capture")
    # A capture ID is consumed before serialization/output succeeds, so a
    # recoverable failure cannot retry the same protocol identifier.
    insert_at = transport_source.find("sentCaptureIDs.insert(captureID)")
    serialize_at = transport_source.find("envelope.toJSONString()")
    if insert_at >= 0 and serialize_at >= 0 and insert_at < serialize_at:
        findings.append("capture ID is marked sent before serialization; retry may be suppressed after failure")
    return findings


def deduplication_findings(transport_source: str) -> list[str]:
    """Check that duplicate capture identifiers are rejected at the transport edge."""
    findings: list[str] = []
    if "sentCaptureIDs" not in transport_source:
        findings.append("no sent capture-ID registry found for deduplication")
    if "sentCaptureIDs.insert(captureID).inserted" not in transport_source:
        findings.append("capture ID is not atomically inserted for duplicate suppression")
    if "return false" not in transport_source or "duplicate capture_id" not in transport_source:
        findings.append("duplicate capture IDs are not explicitly rejected")
    return findings


def cross_transport_findings(store_source: str, bonjour_source: str) -> list[str]:
    """Catch the stdout double-submit path across the two companion transports."""
    findings: list[str] = []
    sends_stdout_first = "CaptureTransport.shared.send(envelope)" in store_source
    then_sends_bonjour = "BonjourTransport.shared.send(json)" in store_source
    bonjour_stdout_fallback = "print(jsonLine)" in bonjour_source and "fflush(stdout)" in bonjour_source
    if sends_stdout_first and then_sends_bonjour and bonjour_stdout_fallback:
        findings.append(
            "capture is sent to stdout before Bonjour fallback, so a disconnected capture is emitted twice"
        )
    return findings


def nearby_transport_findings(bonjour_source: str, info_plist: str) -> list[str]:
    """Gate the companion against the published nearby transport security boundary."""
    findings: list[str] = []
    if "NWParameters.tcp" in bonjour_source:
        findings.append("Bonjour transport uses plaintext TCP; nearby delivery requires paired TLS-PSK")
    if "CryptoKit" not in bonjour_source or "add_pre_shared_key" not in bonjour_source:
        findings.append("Bonjour transport has no TLS-PSK pairing implementation")
    if "pair_id" not in bonjour_source or "proof" not in bonjour_source:
        findings.append("hello payload lacks pair_id/proof required by nearby-v1")
    if "NSLocalNetworkUsageDescription" not in info_plist:
        findings.append("Info.plist lacks NSLocalNetworkUsageDescription for physical-device browsing")
    if "NSBonjourServices" not in info_plist or "_flashtex._tcp" not in info_plist:
        findings.append("Info.plist lacks _flashtex._tcp Bonjour service declaration")
    return findings


def receipt_findings(bonjour_source: str) -> list[str]:
    """Ensure a network acknowledgement is not confused with a durable receipt."""
    findings: list[str] = []
    if 'type_ == "capture_received"' not in bonjour_source:
        findings.append("Bonjour transport does not parse capture_received acknowledgements")
        return findings
    if 'payload["durable"]' not in bonjour_source:
        findings.append("capture_received is accepted without checking durable receipt status")
    elif "durable == true" not in bonjour_source and "durable {" not in bonjour_source:
        findings.append("capture_received durable status is parsed but not required before acknowledgement")
    return findings


def interop_findings(companion_bonjour: str, mac_listener: str) -> list[str]:
    """Compare pinned companion and Mac receiver transport capabilities."""
    findings: list[str] = []
    mac_requires_psk = "sec_protocol_options_add_pre_shared_key" in mac_listener
    mac_requires_hello_proof = "verifyHelloProof" in mac_listener
    companion_has_psk = "add_pre_shared_key" in companion_bonjour
    companion_has_proof = "pair_id" in companion_bonjour and "proof" in companion_bonjour
    if mac_requires_psk and not companion_has_psk:
        findings.append("Mac receiver requires TLS-PSK but companion cannot open a PSK connection")
    if mac_requires_hello_proof and not companion_has_proof:
        findings.append("Mac receiver requires hello pair_id/proof but companion cannot authenticate hello")
    if "NWParameters.tcp" in companion_bonjour and mac_requires_psk:
        findings.append("plaintext companion connection will be rejected before the Mac parses JSON Lines")
    return findings


def fixture_mime_findings(fixture: Path) -> list[str]:
    """Validate declared MIME type against decoded bytes in a capture fixture."""
    try:
        image = json.loads(fixture.read_text(encoding="utf-8"))["payload"]["image"]
        declared = image["mime_type"]
        data = base64.b64decode(image["data_base64"], validate=True)
    except (KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
        return [f"invalid capture fixture: {error}"]
    actual = detected_mime(data)
    if actual is None:
        return ["capture fixture has an unknown image signature"]
    if actual != declared:
        return [f"capture fixture declares {declared} but bytes are {actual}"]
    return []


def available_ios_simulator(command_result: dict[str, Any]) -> str | None:
    """Select an actually installed iOS simulator; never infer one from an SDK."""
    if command_result["exit_code"] != 0:
        return None
    try:
        devices = json.loads(command_result["output"])["devices"]
        for runtime, candidates in devices.items():
            if "iOS" in runtime:
                for device in candidates:
                    if device.get("isAvailable") and device.get("udid"):
                        return device["udid"]
    except (KeyError, TypeError, ValueError):
        pass
    return None


def validation_status(validation: dict[str, Any]) -> dict[str, Any]:
    """Keep source, build, and executed XCTest evidence separate."""
    findings = {key: value for key, value in validation.items() if key.endswith("_findings") and value}
    commands = validation["commands"]
    if not commands or commands[0]["exit_code"] is None:
        xcode_status = "not_run"
    elif commands[0]["exit_code"] != 0 or any(
        item["exit_code"] != 0 for item in commands[1:]
    ):
        xcode_status = "failed"
    elif len(commands) < 4:
        xcode_status = "not_run"
    else:
        xcode_status = "passed"
    return {
        "source": "failed" if findings else "passed",
        "findings": findings,
        "xcode": xcode_status,
        "xctest": validation["xctest"]["status"],
    }


def validate_tree(source: Path, xcodebuild: str, build: bool) -> dict[str, Any]:
    project = source / PROJECT
    payload = source / PAYLOAD
    validator = source / VALIDATOR
    store = source / STORE
    transport = source / TRANSPORT
    bonjour_transport = source / BONJOUR_TRANSPORT
    info_plist = source / INFO_PLIST
    result: dict[str, Any] = {
        "project": str(PROJECT),
        "destination": "sdk: iphonesimulator (direct SDK build; no named simulator required)",
        "pbx_findings": [],
        "test_target_findings": [],
        "mime_findings": [],
        "delivery_findings": [],
        "deduplication_findings": [],
        "cross_transport_findings": [],
        "nearby_transport_findings": [],
        "receipt_findings": [],
        "fixture_mime_findings": [],
        "commands": [],
        "xctest": {"status": "not_run", "reason": "project not loaded"},
    }
    if not project.exists():
        result["pbx_findings"] = ["project.pbxproj missing"]
        return result
    project_text = project.read_text(encoding="utf-8")
    result["pbx_findings"] = pbx_findings(project_text)
    result["test_target_findings"] = target_findings(project_text)
    if payload.exists() and validator.exists():
        result["mime_findings"] = source_mime_findings(
            payload.read_text(encoding="utf-8"),
            validator.read_text(encoding="utf-8"),
            store.read_text(encoding="utf-8") if store.exists() else "",
        )
    if store.exists() and transport.exists():
        result["delivery_findings"] = delivery_findings(
            store.read_text(encoding="utf-8"), transport.read_text(encoding="utf-8")
        )
        result["deduplication_findings"] = deduplication_findings(
            transport.read_text(encoding="utf-8")
        )
        if bonjour_transport.exists():
            result["cross_transport_findings"] = cross_transport_findings(
                store.read_text(encoding="utf-8"), bonjour_transport.read_text(encoding="utf-8")
            )
            result["nearby_transport_findings"] = nearby_transport_findings(
                bonjour_transport.read_text(encoding="utf-8"),
                info_plist.read_text(encoding="utf-8") if info_plist.exists() else "",
            )
            result["receipt_findings"] = receipt_findings(
                bonjour_transport.read_text(encoding="utf-8")
            )
        else:
            result["cross_transport_findings"] = ["Bonjour transport source missing"]
            result["nearby_transport_findings"] = ["Bonjour transport source missing"]
            result["receipt_findings"] = ["Bonjour transport source missing"]
    else:
        result["delivery_findings"] = ["capture delivery sources missing"]
        result["deduplication_findings"] = ["capture delivery sources missing"]
        result["cross_transport_findings"] = ["capture delivery sources missing"]
        result["nearby_transport_findings"] = ["capture delivery sources missing"]
        result["receipt_findings"] = ["capture delivery sources missing"]
    fixture = source / CAPTURE_FIXTURE
    if fixture.exists():
        result["fixture_mime_findings"] = fixture_mime_findings(fixture)
    else:
        result["fixture_mime_findings"] = ["capture fixture missing"]
    project_bundle = project.parent
    result["commands"].append(run([xcodebuild, "-list", "-project", str(project_bundle)]))
    if result["commands"][-1]["exit_code"] == 0:
        result["commands"].append(
            run([xcodebuild, "-showdestinations", "-project", str(project_bundle), "-scheme", "FlashTeXCompanion"])
        )
    if build and result["commands"][-1]["exit_code"] == 0:
        result["commands"].append(
            run(
                [
                    xcodebuild,
                    "-quiet",
                    "-project",
                    str(project_bundle),
                    "-target",
                    "FlashTeXCompanion",
                    "-sdk",
                    "iphonesimulator",
                    "CODE_SIGNING_ALLOWED=NO",
                    "build",
                ],
                cwd=source,
            )
        )
        # XCTest source compilation is a separate gate: it catches an orphaned
        # test target even on hosts with no installed simulator runtime.
        result["commands"].append(
            run(
                [
                    xcodebuild,
                    "-quiet",
                    "-project",
                    str(project_bundle),
                    "-target",
                    "FlashTeXCompanionTests",
                    "-configuration",
                    "Debug",
                    "-sdk",
                    "iphonesimulator",
                    "CODE_SIGNING_ALLOWED=NO",
                    "build",
                ],
                cwd=source,
            )
        )
        if all(command["exit_code"] == 0 for command in result["commands"]):
            simulators = run(["xcrun", "simctl", "list", "devices", "available", "-j"], output_limit=100000)
            simulator_id = available_ios_simulator(simulators)
            if simulator_id:
                test = run([
                    xcodebuild, "-project", str(project_bundle), "-scheme", "FlashTeXCompanion",
                    "-destination", f"platform=iOS Simulator,id={simulator_id}",
                    "CODE_SIGNING_ALLOWED=NO", "test",
                ], cwd=source, timeout=600)
                result["xctest"] = {
                    "status": "passed" if test["exit_code"] == 0 else "failed",
                    "command": test,
                }
            else:
                result["xctest"] = {"status": "not_run", "reason": "no available iOS simulator", "simulator_query": simulators}
        else:
            result["xctest"] = {"status": "not_run", "reason": "project list or target build failed"}
    elif not build:
        result["xctest"] = {"status": "not_run", "reason": "--skip-build requested"}
    else:
        result["xctest"] = {"status": "not_run", "reason": "Xcode project list/destinations unavailable"}
    return result


def repair_patch(repo: Path, bad_ref: str, repair_ref: str) -> bytes:
    return subprocess.run(
        [
            "git", "-C", str(repo), "diff", "--binary", bad_ref, repair_ref,
            "--", "apps/companion/FlashTeXCompanion.xcodeproj/project.pbxproj",
        ],
        capture_output=True,
        check=True,
    ).stdout


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--bad-ref", required=True, help="pinned FT-004 revision")
    parser.add_argument("--repair-ref", help="candidate repair revision")
    parser.add_argument("--mac-ref", help="pinned Mac nearby-listener revision for interop validation")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--xcodebuild", default="xcodebuild")
    parser.add_argument("--skip-build", action="store_true")
    parser.add_argument("--require-native-tests", action="store_true", help="fail unless the repaired/current XCTest scheme executed successfully")
    args = parser.parse_args(argv)

    repo = args.repo.resolve()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    report: dict[str, Any] = {
        "schema_version": 1,
        "bad_ref": args.bad_ref,
        "bad_sha": resolve_revision(repo, args.bad_ref),
        "repair_ref": args.repair_ref,
        "repair_sha": resolve_revision(repo, args.repair_ref) if args.repair_ref else None,
        "mac_ref": args.mac_ref,
        "mac_sha": resolve_revision(repo, args.mac_ref) if args.mac_ref else None,
        "repo": str(repo),
        "xcodebuild": shutil.which(args.xcodebuild) or args.xcodebuild,
        "toolchain": {
            "xcode_version": run([args.xcodebuild, "-version"]),
            "available_sdks": run([args.xcodebuild, "-showsdks"]),
            "simulator_runtimes": run(["xcrun", "simctl", "list", "runtimes"]),
        },
        "validations": {},
    }
    with tempfile.TemporaryDirectory(prefix="flashtex-companion-") as temporary:
        temporary_path = Path(temporary)
        bad_export = temporary_path / "bad"
        bad_export.mkdir()
        export_revision(repo, args.bad_ref, bad_export)
        report["validations"]["pinned"] = validate_tree(
            bad_export / "source", args.xcodebuild, not args.skip_build
        )
        if args.repair_ref:
            repaired_export = temporary_path / "repaired"
            repaired_export.mkdir()
            export_revision(repo, args.repair_ref, repaired_export)
            report["validations"]["repair"] = validate_tree(
                repaired_export / "source", args.xcodebuild, not args.skip_build
            )
            patch = repair_patch(repo, args.bad_ref, args.repair_ref)
            patch_path = output / "companion-project-repair.patch"
            patch_path.write_bytes(patch)
            report["repair_patch_bytes"] = len(patch)
            patched_export = temporary_path / "patched"
            patched_export.mkdir()
            export_revision(repo, args.bad_ref, patched_export)
            patched_source = patched_export / "source"
            initialize = run(["git", "init", "-q"], cwd=patched_source)
            apply = run(["git", "apply", "--binary", str(patch_path)], cwd=patched_source)
            patch_validation: dict[str, Any] = {
                "commands": [initialize, apply],
                "project_matches_repair": False,
            }
            if initialize["exit_code"] == 0 and apply["exit_code"] == 0:
                patch_validation["validation"] = validate_tree(
                    patched_source, args.xcodebuild, not args.skip_build
                )
                patch_validation["project_matches_repair"] = (
                    (patched_source / PROJECT).read_bytes()
                    == (repaired_export / "source" / PROJECT).read_bytes()
                )
            report["patch_validation"] = patch_validation
        if args.mac_ref:
            mac_export = temporary_path / "mac"
            mac_export.mkdir()
            export_revision(repo, args.mac_ref, mac_export)
            companion_source = (
                repaired_export / "source" if args.repair_ref else bad_export / "source"
            )
            companion_bonjour = companion_source / BONJOUR_TRANSPORT
            listener = mac_export / "source" / MAC_NEARBY_LISTENER
            report["interop_validation"] = {
                "companion_source_sha": report["repair_sha"] or report["bad_sha"],
                "mac_source_sha": report["mac_sha"],
                "findings": interop_findings(
                    companion_bonjour.read_text(encoding="utf-8") if companion_bonjour.exists() else "",
                    listener.read_text(encoding="utf-8") if listener.exists() else "",
                ),
                "missing_sources": [
                    str(path) for path in (companion_bonjour, listener) if not path.exists()
                ],
            }
    candidate = report["validations"].get("repair", report["validations"]["pinned"])
    status = validation_status(candidate)
    interop = report.get("interop_validation")
    if interop and (interop["findings"] or interop["missing_sources"]):
        status["interop"] = "failed"
    elif interop:
        status["interop"] = "passed"
    status["overall"] = "failed" if status["source"] == "failed" or status.get("interop") == "failed" or status["xcode"] == "failed" or status["xctest"] == "failed" else "passed_with_native_not_run" if status["xctest"] == "not_run" else "passed"
    report["status"] = status
    (output / "report.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    print(json.dumps(report, indent=2, sort_keys=True))
    if args.require_native_tests and status["xctest"] != "passed":
        return 2
    return 1 if status["overall"] == "failed" else 0


if __name__ == "__main__":
    raise SystemExit(main())
