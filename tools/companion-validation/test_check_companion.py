import importlib.util
from pathlib import Path
import tempfile
import unittest
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
            with patch.object(check_companion, "run", return_value={"exit_code": 0}) as runner:
                result = check_companion.validate_tree(root, "xcodebuild", False)
            self.assertEqual(runner.call_args_list[0].args[0][-1], str(project.parent))
            self.assertEqual(result["destination"], "sdk: iphonesimulator (direct SDK build; no named simulator required)")


if __name__ == "__main__":
    unittest.main()
