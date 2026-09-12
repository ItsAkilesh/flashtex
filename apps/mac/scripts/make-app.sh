#!/usr/bin/env bash
# Packages the FlashTeXMac SwiftPM executable as apps/mac/build/FlashTeX.app.
#
# A bare `swift build` product has no Finder/Dock identity: `open -a` fails on
# it and macOS cannot grant it per-app permissions (e.g. local network). This
# wraps the built executable in a minimal .app bundle so it can be launched
# with `open` and eventually granted such permissions.
#
# Usage: apps/mac/scripts/make-app.sh [--debug] [--compiler <path>] [--pdf <path>]
#                                      [--bridge <path>] [--ledger <path>]
#                                      [--open] [--install] [--dmg]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MAC_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$MAC_DIR/../.." && pwd)"

CONFIG="release"
COMPILER_PATH=""
PDF_PATH=""
BRIDGE_PATH=""
LEDGER_PATH=""
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
    --bridge)
      BRIDGE_PATH="${2:-}"
      shift 2
      ;;
    --ledger)
      LEDGER_PATH="${2:-}"
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
      sed -n '2,13p' "${BASH_SOURCE[0]}"
      exit 0
      ;;
    *)
      echo "make-app.sh: unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

# Resolves the short git SHA of the repo that CONTAINS $1 (the resolved source
# path of a bundled binary, before it is copied into the bundle) — never the
# app repo's own SHA for a binary sourced from a different worktree.
component_git_sha() {
  local dir toplevel
  dir="$(cd "$(dirname "$1")" 2>/dev/null && pwd)" || { echo "unknown"; return; }
  toplevel="$(git -C "$dir" rev-parse --show-toplevel 2>/dev/null)" || { echo "unknown"; return; }
  git -C "$toplevel" rev-parse --short HEAD 2>/dev/null || echo "unknown"
}

COMPONENTS_JSON_ENTRIES=()
# Appends one components.json entry. $1=name $2=bundled-path-in-app-or-empty $3=source-path-or-empty
record_component() {
  local name="$1" bundled_path="$2" source_path="$3"
  if [[ -n "$bundled_path" && -f "$bundled_path" ]]; then
    local sha256 git_sha
    sha256="$(shasum -a 256 "$bundled_path" | awk '{print $1}')"
    git_sha="$(component_git_sha "$source_path")"
    COMPONENTS_JSON_ENTRIES+=("  \"$name\": {\"bundled\": true, \"source_path\": \"$source_path\", \"git_sha\": \"$git_sha\", \"sha256\": \"$sha256\"}")
  else
    COMPONENTS_JSON_ENTRIES+=("  \"$name\": {\"bundled\": false, \"source_path\": null, \"git_sha\": null, \"sha256\": null}")
  fi
}

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
  record_component "compiler" "$MACOS_DIR/flashtex-compiler" "$COMPILER_PATH"
else
  echo "    no flashtex-compiler found (build crates/compiler or pass --compiler <path>); skipping"
  record_component "compiler" "" ""
fi

if [[ -z "$PDF_PATH" ]]; then
  DEFAULT_PDF="$REPO_ROOT/crates/pdf/target/release/flashtex-pdf"
  [[ -f "$DEFAULT_PDF" ]] && PDF_PATH="$DEFAULT_PDF"
fi
if [[ -n "$PDF_PATH" && -f "$PDF_PATH" ]]; then
  cp "$PDF_PATH" "$MACOS_DIR/flashtex-pdf"
  chmod +x "$MACOS_DIR/flashtex-pdf"
  echo "    bundled flashtex-pdf from $PDF_PATH"
  record_component "pdf" "$MACOS_DIR/flashtex-pdf" "$PDF_PATH"
else
  echo "    no flashtex-pdf found (build crates/pdf or pass --pdf <path>); skipping"
  record_component "pdf" "" ""
fi

if [[ -z "$BRIDGE_PATH" ]]; then
  DEFAULT_BRIDGE="$REPO_ROOT/crates/bridge/target/release/flashtex-bridge"
  if [[ ! -f "$DEFAULT_BRIDGE" && -f "$REPO_ROOT/crates/bridge/Cargo.toml" ]]; then
    echo "    flashtex-bridge not built; building it (cargo build --release)…"
    if (cd "$REPO_ROOT/crates/bridge" && cargo build --release); then
      echo "    built flashtex-bridge"
    else
      echo "    warning: cargo build --release failed for crates/bridge" >&2
    fi
  fi
  [[ -f "$DEFAULT_BRIDGE" ]] && BRIDGE_PATH="$DEFAULT_BRIDGE"
fi
if [[ -n "$BRIDGE_PATH" && -f "$BRIDGE_PATH" ]]; then
  cp "$BRIDGE_PATH" "$MACOS_DIR/flashtex-bridge"
  chmod +x "$MACOS_DIR/flashtex-bridge"
  echo "    bundled flashtex-bridge from $BRIDGE_PATH"
  record_component "bridge" "$MACOS_DIR/flashtex-bridge" "$BRIDGE_PATH"
else
  echo "    no flashtex-bridge found (build crates/bridge or pass --bridge <path>); skipping"
  record_component "bridge" "" ""
fi

if [[ -z "$LEDGER_PATH" ]]; then
  DEFAULT_LEDGER="$REPO_ROOT/crates/edit-ledger/target/release/flashtex-edit-ledger"
  [[ -f "$DEFAULT_LEDGER" ]] && LEDGER_PATH="$DEFAULT_LEDGER"
fi
if [[ -n "$LEDGER_PATH" && -f "$LEDGER_PATH" ]]; then
  cp "$LEDGER_PATH" "$MACOS_DIR/flashtex-edit-ledger"
  chmod +x "$MACOS_DIR/flashtex-edit-ledger"
  echo "    bundled flashtex-edit-ledger from $LEDGER_PATH"
  record_component "edit_ledger" "$MACOS_DIR/flashtex-edit-ledger" "$LEDGER_PATH"
else
  echo "    no flashtex-edit-ledger found (build crates/edit-ledger or pass --ledger <path>); skipping"
  record_component "edit_ledger" "" ""
fi

# The app's own entry uses $GIT_SHA (already resolved for the whole repo
# worktree) directly rather than record_component's file-based git lookup,
# which expects a binary path, not a directory.
APP_SHA256="$(shasum -a 256 "$MACOS_DIR/FlashTeX" | awk '{print $1}')"
COMPONENTS_JSON_ENTRIES+=("  \"app\": {\"bundled\": true, \"source_path\": \"$REPO_ROOT\", \"git_sha\": \"$GIT_SHA\", \"sha256\": \"$APP_SHA256\"}")

echo "==> Writing Contents/Resources/components.json"
{
  echo "{"
  LAST_INDEX=$((${#COMPONENTS_JSON_ENTRIES[@]} - 1))
  for i in "${!COMPONENTS_JSON_ENTRIES[@]}"; do
    if [[ "$i" -eq "$LAST_INDEX" ]]; then
      echo "${COMPONENTS_JSON_ENTRIES[$i]}"
    else
      echo "${COMPONENTS_JSON_ENTRIES[$i]},"
    fi
  done
  echo "}"
} > "$RESOURCES_DIR/components.json"
if python3 -c "import json,sys; json.load(open(sys.argv[1]))" "$RESOURCES_DIR/components.json" 2>/dev/null; then
  echo "    components.json written and valid JSON"
else
  echo "    warning: components.json failed JSON validation" >&2
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
