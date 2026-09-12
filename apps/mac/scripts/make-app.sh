#!/usr/bin/env bash
# Packages the FlashTeXMac SwiftPM executable as apps/mac/build/FlashTeX.app.
#
# A bare `swift build` product has no Finder/Dock identity: `open -a` fails on
# it and macOS cannot grant it per-app permissions (e.g. local network). This
# wraps the built executable in a minimal .app bundle so it can be launched
# with `open` and eventually granted such permissions.
#
# Usage: apps/mac/scripts/make-app.sh [--debug] [--compiler <path>] [--pdf <path>]
#                                      [--open] [--install] [--dmg]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MAC_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$MAC_DIR/../.." && pwd)"

CONFIG="release"
COMPILER_PATH=""
PDF_PATH=""
DO_OPEN=0
DO_INSTALL=0
DO_DMG=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --debug)
      CONFIG="debug"
      shift
      ;;
    --compiler)
      COMPILER_PATH="${2:-}"
      shift 2
      ;;
    --pdf)
      PDF_PATH="${2:-}"
      shift 2
      ;;
    --open)
      DO_OPEN=1
      shift
      ;;
    --install)
      DO_INSTALL=1
      shift
      ;;
    --dmg)
      DO_DMG=1
      shift
      ;;
    -h|--help)
      sed -n '2,11p' "${BASH_SOURCE[0]}"
      exit 0
      ;;
    *)
      echo "make-app.sh: unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

echo "==> Building FlashTeXMac (-c $CONFIG) in $MAC_DIR"
(cd "$MAC_DIR" && swift build -c "$CONFIG")
BIN_DIR="$(cd "$MAC_DIR" && swift build -c "$CONFIG" --show-bin-path)"
BUILT_EXECUTABLE="$BIN_DIR/FlashTeXMac"
if [[ ! -x "$BUILT_EXECUTABLE" ]]; then
  echo "make-app.sh: built executable not found at $BUILT_EXECUTABLE" >&2
  exit 1
fi

APP_DIR="$MAC_DIR/build/FlashTeX.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"
SAMPLES_DIR="$RESOURCES_DIR/Samples"

echo "==> Assembling bundle at $APP_DIR"
rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR" "$SAMPLES_DIR"

cp "$BUILT_EXECUTABLE" "$MACOS_DIR/FlashTeX"
chmod +x "$MACOS_DIR/FlashTeX"

GIT_SHA="$(cd "$REPO_ROOT" && git rev-parse --short HEAD 2>/dev/null || echo unknown)"

cat > "$CONTENTS_DIR/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleName</key>
	<string>FlashTeX</string>
	<key>CFBundleDisplayName</key>
	<string>FlashTeX</string>
	<key>CFBundleIdentifier</key>
	<string>tech.jay3332.flashtex.mac</string>
	<key>CFBundleExecutable</key>
	<string>FlashTeX</string>
	<key>CFBundlePackageType</key>
	<string>APPL</string>
	<key>CFBundleShortVersionString</key>
	<string>0.1.0</string>
	<key>CFBundleVersion</key>
	<string>$GIT_SHA</string>
	<key>LSMinimumSystemVersion</key>
	<string>14.0</string>
	<key>NSHighResolutionCapable</key>
	<true/>
	<key>NSPrincipalClass</key>
	<string>NSApplication</string>
	<key>LSApplicationCategoryType</key>
	<string>public.app-category.productivity</string>
	<key>NSLocalNetworkUsageDescription</key>
	<string>FlashTeX uses the local network to discover and receive captures from nearby devices.</string>
	<key>NSBonjourServices</key>
	<array>
		<string>_flashtex._tcp</string>
	</array>
</dict>
</plist>
PLIST

printf 'APPL????' > "$CONTENTS_DIR/PkgInfo"

echo "==> Copying fixtures and samples into Contents/Resources/Samples"
FIXTURES_DIR="$REPO_ROOT/protocol/fixtures"
for f in compile-result.json compile-request.json; do
  if [[ -f "$FIXTURES_DIR/$f" ]]; then
    cp "$FIXTURES_DIR/$f" "$SAMPLES_DIR/"
  else
    echo "make-app.sh: warning: missing fixture $FIXTURES_DIR/$f" >&2
  fi
done
if [[ -d "$MAC_DIR/Samples" ]]; then
  cp -R "$MAC_DIR/Samples/." "$SAMPLES_DIR/"
fi

echo "==> Locating built Rust binaries"
if [[ -z "$COMPILER_PATH" ]]; then
  DEFAULT_COMPILER="$REPO_ROOT/crates/compiler/target/release/flashtex-compiler"
  [[ -f "$DEFAULT_COMPILER" ]] && COMPILER_PATH="$DEFAULT_COMPILER"
fi
if [[ -n "$COMPILER_PATH" && -f "$COMPILER_PATH" ]]; then
  cp "$COMPILER_PATH" "$MACOS_DIR/flashtex-compiler"
  chmod +x "$MACOS_DIR/flashtex-compiler"
  echo "    bundled flashtex-compiler from $COMPILER_PATH"
else
  echo "    no flashtex-compiler found (build crates/compiler or pass --compiler <path>); skipping"
fi

if [[ -z "$PDF_PATH" ]]; then
  DEFAULT_PDF="$REPO_ROOT/crates/pdf/target/release/flashtex-pdf"
  [[ -f "$DEFAULT_PDF" ]] && PDF_PATH="$DEFAULT_PDF"
fi
if [[ -n "$PDF_PATH" && -f "$PDF_PATH" ]]; then
  cp "$PDF_PATH" "$MACOS_DIR/flashtex-pdf"
  chmod +x "$MACOS_DIR/flashtex-pdf"
  echo "    bundled flashtex-pdf from $PDF_PATH"
else
  echo "    no flashtex-pdf found (build crates/pdf or pass --pdf <path>); skipping"
fi

echo "==> Ad-hoc codesigning"
if command -v codesign >/dev/null 2>&1; then
  if codesign --force --deep --sign - "$APP_DIR"; then
    echo "    codesign OK (ad-hoc)"
  else
    echo "    warning: codesign failed; bundle is unsigned" >&2
  fi
else
  echo "    warning: codesign not available on this system; bundle is unsigned" >&2
fi

if [[ "$DO_DMG" -eq 1 ]]; then
  echo "==> Building DMG"
  DMG_PATH="$MAC_DIR/build/FlashTeX.dmg"
  DMG_STAGING="$(mktemp -d "${TMPDIR:-/tmp}/flashtex-dmg.XXXXXX")"
  cp -R "$APP_DIR" "$DMG_STAGING/FlashTeX.app"
  ln -s /Applications "$DMG_STAGING/Applications"
  rm -f "$DMG_PATH"
  if hdiutil create -volname "FlashTeX" -srcfolder "$DMG_STAGING" -ov -format UDZO "$DMG_PATH" >/dev/null; then
    echo "    DMG at $DMG_PATH"
  else
    echo "    warning: hdiutil failed; no DMG produced" >&2
  fi
  rm -rf "$DMG_STAGING"
fi

if [[ "$DO_INSTALL" -eq 1 ]]; then
  echo "==> Installing to ~/Applications"
  INSTALL_DIR="$HOME/Applications"
  DEST="$INSTALL_DIR/FlashTeX.app"
  PREVIOUS="$INSTALL_DIR/FlashTeX-previous.app"
  STAGED="$INSTALL_DIR/.FlashTeX.app.staging.$$"
  mkdir -p "$INSTALL_DIR"
  rm -rf "$STAGED"

  # Copy into a hidden staging name first, then rename (same directory, so
  # the final `mv` is a single atomic rename): $DEST never briefly points at
  # a half-copied bundle.
  cp -R "$APP_DIR" "$STAGED"
  if [[ -d "$DEST" ]]; then
    rm -rf "$PREVIOUS"
    mv "$DEST" "$PREVIOUS"
    echo "    kept previous install at $PREVIOUS pending launch verification"
  fi
  mv "$STAGED" "$DEST"
  echo "    installed $DEST"

  echo "==> Verifying the installed app launches"
  pkill -x FlashTeX >/dev/null 2>&1 || true
  sleep 1
  open "$DEST"
  LAUNCHED=0
  for _ in $(seq 1 10); do
    if pgrep -x FlashTeX >/dev/null 2>&1; then
      LAUNCHED=1
      break
    fi
    sleep 1
  done

  if [[ "$LAUNCHED" -eq 1 ]]; then
    echo "    launch OK (FlashTeX process is running)"
    osascript -e 'tell application "FlashTeX" to quit' >/dev/null 2>&1 || pkill -x FlashTeX >/dev/null 2>&1 || true
    if [[ -d "$PREVIOUS" ]]; then
      rm -rf "$PREVIOUS"
      echo "    removed $PREVIOUS (new install verified)"
    fi
  else
    echo "    warning: installed app did not launch within 10s; rolling back" >&2
    rm -rf "$DEST"
    if [[ -d "$PREVIOUS" ]]; then
      mv "$PREVIOUS" "$DEST"
      echo "    restored previous install at $DEST" >&2
    fi
    exit 1
  fi
fi

echo "==> Done: $APP_DIR"
echo "    open \"$APP_DIR\""

if [[ "$DO_OPEN" -eq 1 ]]; then
  open "$APP_DIR"
fi
