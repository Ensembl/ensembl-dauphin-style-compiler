import unittest
from typing import Any, cast

from command.controlcmds import ExpansionHandler
from model.expansions import ExpansionMetadataError, Expansions
from model.trackapi import TrackApiError


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
        expansion._profiles = {}
        expansion._templates = {}
        return expansion

    def test_generated_track_keeps_track_api_filepath(self):
        filepath = "tracks/track-uuid/variants.bb"
        tracks = self.expansion_with_payload(track_payload(filepath)).register_track("track-uuid")
        track = tracks._tracks["track-uuid-variant-summary"]

        self.assertIn(("datafile", filepath), track._values)
        self.assertIn(("track", "expand", "track-uuid"), track._triggers)

    def test_invalid_filepath_rejects_the_whole_expansion(self):
        expansion = self.expansion_with_payload(track_payload("../variants.bb"))

        with self.assertRaisesRegex(ExpansionMetadataError, "Invalid datafile path"):
            expansion.register_track("track-uuid")

    def test_gene_profile_uses_one_datafile_for_all_legacy_programs(self):
        payload = {
            "track_id": "gene-uuid",
            "trigger": ["track", "gene-pc-fwd"],
            "label": "Protein coding genes",
            "display_order": 1,
            "datafiles": {"gene": "tracks/gene-uuid/genes.bb"},
            "settings": {},
        }
        expansion = self.expansion_with_payload(payload)
        expansion._profiles = {
            ("track", "gene-pc-fwd"): {
                "template": "gene",
                "variables": {"datafile_key": "gene", "gene_setting": "pc-fwd"},
            }
        }
        expansion._templates = {
            "gene": {
                "track": [
                    {
                        "id": f"gene-{index}",
                        "program": "gene",
                        "datafile_key": "{datafile_key}",
                        "scales": [19, 22, 4],
                        "settings": {"{gene_setting}": ""},
                    }
                    for index in range(5)
                ]
            }
        }

        tracks = expansion.register_track("gene-uuid")

        self.assertEqual(len(tracks._tracks), 5)
        for track in tracks._tracks.values():
            self.assertIn(("datafile", "tracks/gene-uuid/genes.bb"), track._values)
            self.assertIn(("track", "expand", "gene-uuid"), track._triggers)
            self.assertIn(("pc-fwd", ("track", "expand", "gene-uuid")), track._settings)

    def test_gene_fallback_template_keeps_expanded_buttons_setting(self):
        _, templates = Expansions._load_profiles()

        for track in templates["gene"]["track"]:
            self.assertEqual(track["settings"]["expanded"], ["buttons"])

    def test_fixed_trigger_without_a_profile_is_rejected(self):
        payload = track_payload("tracks/track-uuid/variants.bb")
        payload["trigger"] = ["track", "legacy-track"]

        with self.assertRaisesRegex(ExpansionMetadataError, "No fallback profile"):
            self.expansion_with_payload(payload).register_track("track-uuid")


class ExpansionHandlerTests(unittest.TestCase):
    def test_track_api_error_is_returned_as_an_expansion_error(self):
        class FailingExpansions:
            def register_track(self, _track_id):
                raise TrackApiError("Track API request failed")

        class BootTracks:
            @staticmethod
            def get_expansion(_name):
                class Expansion:
                    @staticmethod
                    def callback():
                        return "register_track"

                return Expansion()

        class DataAccessor:
            boot_tracks = {16: BootTracks()}

        class Version:
            @staticmethod
            def get_egs():
                return 16

        response = ExpansionHandler(FailingExpansions()).process(
            cast(Any, DataAccessor()),
            ("ensembl", "main"),
            ("general", "contig"),
            cast(Any, None),
            cast(Any, Version()),
        )

        self.assertEqual(response.payload, b"\x82\x01x\x18Track API request failed")
