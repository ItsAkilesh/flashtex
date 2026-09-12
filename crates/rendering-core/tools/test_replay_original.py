"""Decision/setup regressions; synthetic inputs are not visual acceptance evidence."""
import tempfile
from pathlib import Path
import unittest
import replay_original as replay

class Gates(unittest.TestCase):
    def outcome(self, expected, *args):
        with self.assertRaises(replay.Outcome) as caught:
            replay.decide_measurement(*args)
        self.assertEqual(caught.exception.code, expected)

    def test_matching_raster_does_not_hide_incomplete_reference_text(self):
        self.outcome(4, 0, True, "oracle_limitation")
        self.outcome(4, 0, False, "oracle_limitation")

    def test_unknown_text_policy_is_not_accepted(self):
        self.outcome(4, 0, True, "new_unreviewed_policy")

    def test_raster_mismatch_survives_reference_text_limitation(self):
        self.outcome(3, 1244, False, "oracle_limitation")

    def test_limited_text_mismatch_is_separate(self):
        self.outcome(3, 0, False, "limited_linear_text")
        replay.decide_measurement(0, True, "limited_linear_text")

    def test_digest_drift_and_missing_assets_are_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            asset = Path(directory) / "font"
            with self.assertRaises(replay.Outcome) as caught:
                replay.pinned(asset, replay.sha(b"expected"))
            self.assertEqual(caught.exception.code, 2)
            asset.write_bytes(b"replacement")
            with self.assertRaises(replay.Outcome) as caught:
                replay.pinned(asset, replay.sha(b"expected"))
            self.assertEqual(caught.exception.code, 2)

    def test_fixture_manifest_itself_is_pinned(self):
        replay.pinned(Path(replay.__file__).with_name("replay-fixtures.json"), replay.FIXTURE_MANIFEST_SHA256)

if __name__ == "__main__":
    unittest.main()
