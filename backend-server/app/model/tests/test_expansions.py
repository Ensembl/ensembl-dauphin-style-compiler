import unittest

from model.expansions import ExpansionMetadataError, Expansions


def track_payload(filepath):
    return {
        "track_id": "track-uuid",
        "trigger": ["track", "expand", "track-uuid"],
        "label": "Example track",
        "display_order": 1,
        "datafiles": {"variant-summary": filepath},
        "settings": {"variant-summary": {"scales": [6, 100, 4]}},
    }


class ExpansionTrackFilepathTests(unittest.TestCase):
    def expansion_with_payload(self, payload):
        expansion = Expansions.__new__(Expansions)
        expansion._get_track_data = lambda _track_id: payload
        return expansion

    def test_generated_track_keeps_track_api_filepath(self):
        filepath = "tracks/track-uuid/variants.bb"
        tracks = self.expansion_with_payload(track_payload(filepath)).register_track("track-uuid")
        track = tracks._tracks["track-uuid-variant-summary"]

        self.assertIn(("datafile", filepath), track._values)

    def test_invalid_filepath_rejects_the_whole_expansion(self):
        expansion = self.expansion_with_payload(track_payload("../variants.bb"))

        with self.assertRaisesRegex(ExpansionMetadataError, "Invalid datafile path"):
            expansion.register_track("track-uuid")
