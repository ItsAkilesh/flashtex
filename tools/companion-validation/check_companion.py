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
CAPTURE_FIXTURE = Path("protocol/fixtures/capture-submission.json")
PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"
JPEG_SIGNATURE = b"\xff\xd8\xff"


def run(command: list[str], cwd: Path | None = None, timeout: int = 120) -> dict[str, Any]:
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
            "output": output[-4000:],
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


def source_mime_findings(payload_source: str, validator_source: str) -> list[str]:
    findings: list[str] = []
    payload_png = "pngData()" in payload_source and 'mimeType: "image/png"' in payload_source
    validator_can_jpeg = "jpegData(" in validator_source
    if validator_can_jpeg and payload_png:
        findings.append("validator may produce JPEG but envelope always serializes PNG with image/png")
    if "acceptedMIMETypes" in validator_source and "image/jpeg" in validator_source and not validator_can_jpeg:
        findings.append("JPEG declared accepted but no JPEG encoding path found")
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


def validate_tree(source: Path, xcodebuild: str, build: bool) -> dict[str, Any]:
    project = source / PROJECT
    payload = source / PAYLOAD
    validator = source / VALIDATOR
    result: dict[str, Any] = {
        "project": str(PROJECT),
        "destination": "sdk: iphonesimulator (direct SDK build; no named simulator required)",
        "pbx_findings": [],
        "test_target_findings": [],
        "mime_findings": [],
        "fixture_mime_findings": [],
        "commands": [],
    }
    if not project.exists():
        result["pbx_findings"] = ["project.pbxproj missing"]
        return result
    project_text = project.read_text(encoding="utf-8")
    result["pbx_findings"] = pbx_findings(project_text)
    result["test_target_findings"] = target_findings(project_text)
    if payload.exists() and validator.exists():
        result["mime_findings"] = source_mime_findings(
            payload.read_text(encoding="utf-8"), validator.read_text(encoding="utf-8")
        )
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
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--xcodebuild", default="xcodebuild")
    parser.add_argument("--skip-build", action="store_true")
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
    (output / "report.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
