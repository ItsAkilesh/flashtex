#!/bin/bash
set -e
cd "$(dirname "$0")"
echo "Building FlashTeX Companion..."
xcodebuild -project FlashTeXCompanion.xcodeproj \
  -scheme FlashTeXCompanion \
  -destination 'platform=iOS Simulator,name=iPhone 16' \
  -configuration Debug \
  build 2>&1 | tail -20
echo ""
echo "Build result: $?"
