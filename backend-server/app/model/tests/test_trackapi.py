import unittest
from unittest.mock import patch

from model.trackapi import TrackApiError, TrackDatafileResolver


class FakeTrackApiClient:
    def __init__(self, categories, tracks):
        self.categories = categories
        self.tracks = tracks
        self.category_calls = 0
        self.track_calls = []

    def get_track_categories(self, _genome_id):
        self.category_calls += 1
        return self.categories

    def get_track(self, track_id):
        self.track_calls.append(track_id)
        return self.tracks[track_id]


class TrackDatafileResolverTests(unittest.TestCase):
    def resolver(self, categories, tracks):
        client = FakeTrackApiClient(categories, tracks)
        with patch("model.trackapi.TrackApiClient", return_value=client):
            resolver = TrackDatafileResolver()
        return resolver, client

    def test_focus_gene_uses_the_first_gene_track(self):
        categories = [{"track_list": [
            {"track_id": "gene-first", "type": "gene"},
            {"track_id": "gene-second", "type": "gene"},
        ]}]
        tracks = {
            "gene-first": {"datafiles": {"gene": "598/genome/first.bb"}},
            "gene-second": {"datafiles": {"gene": "598/genome/second.bb"}},
        }
        resolver, client = self.resolver(categories, tracks)

        self.assertEqual(resolver.datafile_for_endpoint("genome", "gene"), "598/genome/first.bb")
        self.assertEqual(resolver.datafile_for_endpoint("genome", "transcript"), "598/genome/first.bb")
        self.assertEqual(client.category_calls, 1)
        self.assertEqual(client.track_calls, ["gene-first"])

    def test_fixed_switch_track_uses_its_exact_trigger(self):
        categories = [{"track_list": [
            {"track_id": "other-regular", "trigger": ["track", "other"]},
            {"track_id": "gc", "trigger": ["track", "gc"]},
        ]}]
        tracks = {
            "gc": {"datafiles": {"gc": "598/genome/gc.bw"}},
            "other-regular": {"datafiles": {"gc": "wrong/path.bw"}},
        }
        resolver, _client = self.resolver(categories, tracks)

        self.assertEqual(resolver.datafile_for_endpoint("genome", "gc"), "598/genome/gc.bw")

    def test_unsafe_track_api_path_is_rejected(self):
        categories = [{"track_list": [{"track_id": "variant", "type": "variant"}]}]
        tracks = {"variant": {"datafiles": {"variant-details": "../variants.bb"}}}
        resolver, _client = self.resolver(categories, tracks)

        with self.assertRaisesRegex(TrackApiError, "Invalid datafile path"):
            resolver.datafile_for_endpoint("genome", "variant-details")
