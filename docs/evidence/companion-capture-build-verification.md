# Build verification: `agent/aarush-macbook/companion-capture`

Verifier: mac-claude-a (Claude Code subagent on mac-m1max-a), 2026-09-12 ~04:50Z.
Requested by aarush-macbook (cannot run xcodebuild in their sandbox).
Verified SHA: `b9d9b3c546cb3dd01f3b49fa9355120a147ef05a`. Nothing on that branch was modified.
Toolchain: Xcode 26.3 (17C529), iOS 26.3 simulator runtime, Darwin 25.3.0.

## Result: FAILS as committed; Swift sources compile after a project-file repair

`xcodebuild -list -project apps/companion/FlashTeXCompanion.xcodeproj` fails before
compiling:

```
xcodebuild: error: Unable to read project 'FlashTeXCompanion.xcodeproj'.
Reason: The project 'FlashTeXCompanion' is damaged and cannot be opened due to a parse error.
```

Cause (from `project.pbxproj`): a scripted edit spliced full object definitions
inline into list positions — a stray `A5000006 /* Services */ = { … }` group body
and `A200000B/C/D` PBXFileReference definitions are duplicated inside the root
group's `children`, the app group's `children`, the Views group's `children`, the
Services group, `productRefGroup` in PBXProject (should be `A5000005 /* Products */`),
the Resources phase `files`, and the Sources phase `files` (ImageValidator listed
three times). Bisect: `5853ffc` parses; `8837135` and `eb1c2fc` each introduced
corruption. The branch has been unbuildable since the second companion commit.

Diagnostic repair (worktree only, not committed; 8 insertions, 42 deletions removing
the spliced fragments and fixing `productRefGroup`), then:

```
xcodebuild -project FlashTeXCompanion.xcodeproj -scheme FlashTeXCompanion \
  -destination 'generic/platform=iOS Simulator' CODE_SIGNING_ALLOWED=NO build
→ ** BUILD SUCCEEDED **, zero compiler errors/warnings
```

## Tests: not runnable

`FlashTeXCompanionTests/` (CapturePayloadTests ×5, ImageValidatorTests ×3) has no
test target, XCTest reference, or `.xcscheme` in the pbxproj; `Package.swift` has
no test target either. `xcrun swiftc -typecheck` of both test files against the
built module exits 0, so they reference real APIs, but no test was executed.

`build.sh`/README hardcode `name=iPhone 16`, which does not exist on Xcode 26.3
(iPhone 17 series and 16e are available); use `generic/platform=iOS Simulator`.

## Contract check vs runtime-v1 `capture_submit`

Envelope and payload keys match exactly (`protocol_version` 1, `type`
`capture_submit`, `capture_id`, `destination_id`, `base_revision`,
`image{mime_type,data_base64}`, `instructions`); one JSON object per line; duplicate
`capture_id` suppressed. Caveats:

1. `mime_type` is hardcoded `image/png` and `CaptureEnvelope.create` always uses
   `pngData()`; `ImageValidator`'s JPEG/10 MB fallback never reaches the wire.
2. `JSONEncoder` lacks `.withoutEscapingSlashes`, so base64 `/` is emitted as `\/`
   (valid JSON, not byte-identical to the fixture despite the code comment).
3. `destination_id`/`base_revision` are user-settable values, not Mac-pinned anchors
   yet (expected at this stage; the Mac shell now pins anchors — see apps/mac).

## Action for aarush-macbook

Regenerate or repair `project.pbxproj` (remove duplicated inline object definitions,
fix `productRefGroup`, dedupe Sources), add a real test target if the tests should
count, and update `build.sh`/README away from the nonexistent `iPhone 16`.
Re-request verification from mac-m1max-a after pushing.
