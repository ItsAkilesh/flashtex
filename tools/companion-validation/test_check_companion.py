import importlib.util
from pathlib import Path
import tempfile
import unittest
import json
from unittest.mock import patch


MODULE = Path(__file__).with_name("check_companion.py")
SPEC = importlib.util.spec_from_file_location("check_companion", MODULE)
check_companion = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(check_companion)


class CompanionValidationTests(unittest.TestCase):
    def test_detects_inline_project_object(self):
        invalid = "children = (\nA1 /* Foo */ = { isa = PBXGroup; };\n);"
        self.assertEqual(
            check_companion.pbx_findings(invalid),
            ["inline PBX object definition inside children list"],
        )

    def test_accepts_reference_only_lists(self):
        valid = "children = (\nA1 /* Foo */,\n);\nproductRefGroup = A2 /* Products */;"
        self.assertEqual(check_companion.pbx_findings(valid), [])

    def test_requires_test_target(self):
        no_tests = 'isa = PBXNativeTarget;\nname = FlashTeXCompanion;'
        self.assertEqual(check_companion.target_findings(no_tests), ["no XCTest PBXNativeTarget found"])
        with_tests = no_tests + '\nisa = PBXNativeTarget;\nname = FlashTeXCompanionTests;'
        self.assertEqual(check_companion.target_findings(with_tests), [])

    def test_detects_png_and_jpeg_bytes(self):
        self.assertEqual(check_companion.detected_mime(b"\x89PNG\r\n\x1a\nbody"), "image/png")
        self.assertEqual(check_companion.detected_mime(b"\xff\xd8\xffbody"), "image/jpeg")
        self.assertIsNone(check_companion.detected_mime(b"not-an-image"))

    def test_flags_jpeg_envelope_mismatch(self):
        findings = check_companion.source_mime_findings(
            'image.pngData()\n mimeType: "image/png"',
            'image.jpegData(compressionQuality: 0.85)',
        )
        self.assertEqual(findings, ["validator may produce JPEG but envelope always serializes PNG with image/png"])

    def test_requires_cancellation_retry_and_safe_retry_id(self):
        findings = check_companion.delivery_findings(
            "func addCapture() {}",
            "let isNew = sentCaptureIDs.insert(captureID).inserted\n"
            "guard let json = envelope.toJSONString() else { return false }",
        )
        self.assertEqual(
            findings,
            [
                "no cancellation API found for an in-flight capture",
                "no retry API found for a failed capture",
                "capture ID is marked sent before serialization; retry may be suppressed after failure",
            ],
        )

    def test_accepts_explicit_recovery_apis_and_post_serialization_tracking(self):
        findings = check_companion.delivery_findings(
            "func cancelCapture() {}\nfunc retryCapture() {}",
            "guard let json = envelope.toJSONString() else { return false }\n"
            "let isNew = sentCaptureIDs.insert(captureID).inserted",
        )
        self.assertEqual(findings, [])

    def test_requires_atomic_capture_id_deduplication(self):
        self.assertEqual(
            check_companion.deduplication_findings("func send() { print(\"sent\") }"),
            [
                "no sent capture-ID registry found for deduplication",
                "capture ID is not atomically inserted for duplicate suppression",
                "duplicate capture IDs are not explicitly rejected",
            ],
        )

    def test_detects_stdout_double_submit_between_transports(self):
        self.assertEqual(
            check_companion.cross_transport_findings(
                "CaptureTransport.shared.send(envelope)\nBonjourTransport.shared.send(json)",
                "print(jsonLine)\nfflush(stdout)",
            ),
            [
                "capture is sent to stdout before Bonjour fallback, so a disconnected capture is emitted twice"
            ],
        )
        self.assertEqual(
            check_companion.cross_transport_findings(
                "BonjourTransport.shared.send(json)", "print(jsonLine)\nfflush(stdout)"
            ),
            [],
        )
        self.assertEqual(
            check_companion.deduplication_findings(
                "sentCaptureIDs.insert(captureID).inserted\n"
                "warning: duplicate capture_id\nreturn false"
            ),
            [],
        )

    def test_validates_declared_fixture_mime_against_bytes(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = Path(temporary) / "capture.json"
            fixture.write_text(json.dumps({"payload": {"image": {
                "mime_type": "image/jpeg",
                "data_base64": "iVBORw0KGgo="
            }}}))
            self.assertEqual(
                check_companion.fixture_mime_findings(fixture),
                ["capture fixture declares image/jpeg but bytes are image/png"],
            )

    def test_uses_xcodeproj_bundle_for_xcodebuild(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            project = root / check_companion.PROJECT
            project.parent.mkdir(parents=True)
            project.write_text("isa = PBXNativeTarget;\nname = FlashTeXCompanion;")
            payload = root / check_companion.PAYLOAD
            validator = root / check_companion.VALIDATOR
            payload.parent.mkdir(parents=True)
            validator.parent.mkdir(parents=True)
            payload.write_text('image.pngData()\n mimeType: "image/png"')
            validator.write_text("")
            store = root / check_companion.STORE
            transport = root / check_companion.TRANSPORT
            bonjour = root / check_companion.BONJOUR_TRANSPORT
            store.parent.mkdir(parents=True, exist_ok=True)
            transport.parent.mkdir(parents=True, exist_ok=True)
            bonjour.parent.mkdir(parents=True, exist_ok=True)
            store.write_text("func cancelCapture() {}\nfunc retryCapture() {}")
            transport.write_text("let json = envelope.toJSONString()\nsentCaptureIDs.insert(captureID)")
            bonjour.write_text("func send() {}")
            with patch.object(check_companion, "run", return_value={"exit_code": 0}) as runner:
                result = check_companion.validate_tree(root, "xcodebuild", False)
            self.assertEqual(runner.call_args_list[0].args[0][-1], str(project.parent))
            self.assertEqual(result["destination"], "sdk: iphonesimulator (direct SDK build; no named simulator required)")


if __name__ == "__main__":
    unittest.main()
