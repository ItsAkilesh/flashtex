#!/bin/bash
set -e
cd "$(dirname "$0")"
echo "Building FlashTeX Companion..."
xcodebuild -project FlashTeXCompanion.xcodeproj \
  -target FlashTeXCompanion \
  -sdk iphonesimulator \
  -configuration Debug \
  CODE_SIGNING_ALLOWED=NO \
  build 2>&1 | tail -20
echo ""
echo "Build result: $?"
