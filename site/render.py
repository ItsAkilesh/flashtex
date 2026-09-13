#!/usr/bin/env python3
"""Render the FlashTeX website for a GitHub release.

    python3 site/render.py OUT_DIR [--tag vX.Y.Z]

Copies site/ into OUT_DIR, filling {{TAG}}, {{DATE}}, {{SIZE}} and {{SHA256}}
in the templated files from the release's FlashTeX.dmg. Without --tag it uses
the latest published release (drafts and prereleases are ignored). Needs an
authenticated `gh`; in GitHub Actions, GH_TOKEN is enough.
"""

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from datetime import datetime
from pathlib import Path
from zoneinfo import ZoneInfo

ASSET = "FlashTeX.dmg"
TEMPLATED = ("index.html", "download/index.html", "install.sh")
NOT_PUBLISHED = {"render.py", "README.md"}


def gh_api(endpoint):
    result = subprocess.run(["gh", "api", endpoint], check=True, capture_output=True, text=True)
    return json.loads(result.stdout)


def find_release(repo, tag, wait_seconds):
    endpoint = f"repos/{repo}/releases/tags/{tag}" if tag else f"repos/{repo}/releases/latest"
    deadline = time.monotonic() + wait_seconds
    while True:
        release = gh_api(endpoint)
        asset = next(
            (a for a in release["assets"] if a["name"] == ASSET and a["state"] == "uploaded"),
            None,
        )
        if asset:
            return release, asset
        # A `release: published` event can arrive before the DMG finishes uploading.
        if time.monotonic() >= deadline:
            sys.exit(f"error: {release['tag_name']} has no uploaded {ASSET} after {wait_seconds}s")
        print(f"Waiting for {ASSET} on {release['tag_name']}...", file=sys.stderr)
        time.sleep(10)


def sha256_of(repo, release, asset):
    digest = asset.get("digest") or ""
    if digest.startswith("sha256:"):
        return digest.split(":", 1)[1]
    with tempfile.TemporaryDirectory() as tmp:
        subprocess.run(
            ["gh", "release", "download", release["tag_name"], "--repo", repo,
             "--pattern", ASSET, "--dir", tmp],
            check=True,
        )
        sha = hashlib.sha256()
        with open(Path(tmp) / ASSET, "rb") as dmg:
            for chunk in iter(lambda: dmg.read(1 << 20), b""):
                sha.update(chunk)
        return sha.hexdigest()


def main():
    parser = argparse.ArgumentParser(description="Render the FlashTeX site for a release.")
    parser.add_argument("out", help="directory to write the rendered site into")
    parser.add_argument("--tag", help="release tag to link (default: latest release)")
    parser.add_argument("--repo", default=os.environ.get("GITHUB_REPOSITORY", "flash-tex/flashtex"))
    parser.add_argument("--timezone", default=os.environ.get("SITE_TIMEZONE", "America/New_York"),
                        help="timezone used for the displayed release date")
    parser.add_argument("--wait", type=int, default=300,
                        help="seconds to wait for the DMG to finish uploading")
    args = parser.parse_args()

    release, asset = find_release(args.repo, args.tag, args.wait)
    published = datetime.fromisoformat(release["published_at"].replace("Z", "+00:00"))
    published = published.astimezone(ZoneInfo(args.timezone))
    values = {
        "TAG": release["tag_name"],
        "DATE": f"{published:%B} {published.day}, {published.year}",
        "SIZE": f"{asset['size'] / (1024 * 1024):.1f} MB",
        "SHA256": sha256_of(args.repo, release, asset),
    }

    source = Path(__file__).resolve().parent
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    for path in sorted(source.rglob("*")):
        relative = path.relative_to(source)
        if path.is_dir() or relative.name in NOT_PUBLISHED or "__pycache__" in relative.parts:
            continue
        destination = out / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        if relative.as_posix() in TEMPLATED:
            text = path.read_text(encoding="utf-8")
            for key, value in values.items():
                text = text.replace("{{" + key + "}}", value)
            if "{{" in text:
                sys.exit(f"error: unfilled placeholder left in {relative}")
            destination.write_text(text, encoding="utf-8")
        else:
            shutil.copy2(path, destination)
    (out / ".nojekyll").touch()
    print(json.dumps(values))


if __name__ == "__main__":
    main()
