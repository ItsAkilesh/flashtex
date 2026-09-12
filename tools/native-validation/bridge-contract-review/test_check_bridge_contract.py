import hashlib
import unittest

from check_bridge_contract import review, verify_edit


class ScalarGuardTests(unittest.TestCase):
    def setUp(self):
        self.text = "aé😀z"
        self.edit = dict(project_id="demo", path="main.tex", expected_revision=3,
                         document_before_sha256=hashlib.sha256(self.text.encode()).hexdigest(),
                         start_byte=1, end_byte=7, removed_text="é😀", replacement="x")

    def verify(self, **overrides):
        return verify_edit(self.edit | overrides, project="demo", path="main.tex", revision=3, text=self.text)

    def test_valid_multibyte_replacement(self):
        self.assertEqual(self.verify(), "axz")

    def test_each_identity_guard(self):
        for key, value in [("project_id", "other"), ("path", "other.tex"), ("expected_revision", 4),
                           ("document_before_sha256", "0" * 64), ("removed_text", "é")]:
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.verify(**{key: value})

    def test_all_non_scalar_offsets_refused(self):
        for offset in [2, 4, 5, 6]:
            for field in ["start_byte", "end_byte"]:
                with self.subTest(offset=offset, field=field), self.assertRaises(ValueError):
                    self.verify(**{field: offset})

    def test_out_of_range_and_inverted_offsets_refused(self):
        for start, end in [(-1, 7), (1, 9), (7, 1), (True, 7)]:
            with self.subTest(start=start, end=end), self.assertRaises(ValueError):
                self.verify(start_byte=start, end_byte=end)

    def test_same_length_changed_source_refused(self):
        with self.assertRaises(ValueError):
            verify_edit(self.edit, project="demo", path="main.tex", revision=3, text="bé😀z")


class StructuralSignalsTests(unittest.TestCase):
    def signals(self, session="", client="", shell=""):
        return {s["check"]: s for s in review({"BridgeSession.swift": session, "BridgeClient.swift": client,
                                              "ShellModel+Bridge.swift": shell})}

    def test_receipt_fallthrough_detected_and_return_not_flagged(self):
        source = '''    func applicationApplied() {
        do { try ledger.update() } catch {
            status = "failure"
        }
        sendApplied()
    }
'''
        self.assertEqual(self.signals(session=source)["receipt_after_ledger_write_failure"]["status"], "FAIL")
        self.assertNotIn("receipt_after_ledger_write_failure", self.signals(session=source.replace('status = "failure"', "return")))

    def test_missing_source_guards_do_not_pass(self):
        self.assertEqual(self.signals()["prepared_source_guards_present"]["status"], "REVIEW")

    def test_plain_pipe_write_flagged(self):
        source = '''    func send() {
        try stdin.fileHandleForWriting.write(contentsOf: line)
    }
'''
        self.assertEqual(self.signals(client=source)["synchronous_bridge_pipe_write"]["status"], "FAIL")


if __name__ == "__main__":
    unittest.main()
