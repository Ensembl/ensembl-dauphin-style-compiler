import unittest
from unittest.mock import patch

import requests

from model.trackapi import TrackApiClient, TrackApiError, TrackDatafileResolver


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

    def test_non_focus_endpoints_are_not_resolved(self):
        categories = [{"track_list": [
            {"track_id": "other-regular", "trigger": ["track", "other"]},
            {"track_id": "gc", "trigger": ["track", "gc"]},
        ]}]
        tracks = {
            "gc": {"datafiles": {"gc": "598/genome/gc.bw"}},
            "other-regular": {"datafiles": {"gc": "wrong/path.bw"}},
        }
        resolver, _client = self.resolver(categories, tracks)

        self.assertIsNone(resolver.datafile_for_endpoint("genome", "gc"))

    def test_unsafe_track_api_path_is_rejected(self):
        categories = [{"track_list": [{"track_id": "variant", "type": "variant"}]}]
        tracks = {"variant": {"datafiles": {"variant-details": "../variants.bb"}}}
        resolver, _client = self.resolver(categories, tracks)

        with self.assertRaisesRegex(TrackApiError, "Invalid datafile path"):
            resolver.datafile_for_endpoint("genome", "variant-details")


class TrackApiClientTests(unittest.TestCase):
    def client(self):
        client = TrackApiClient.__new__(TrackApiClient)
        client._host = "https://track-api.example"
        return client

    def test_track_request_transport_error_is_normalized(self):
        with patch(
            "model.trackapi.requests.get",
            side_effect=requests.exceptions.Timeout("request timed out"),
        ):
            with self.assertRaisesRegex(
                TrackApiError,
                "Track API request failed for track 'track-id': request timed out",
            ):
                self.client().get_track("track-id")

    def test_track_categories_transport_error_is_normalized(self):
        with patch(
            "model.trackapi.requests.get",
            side_effect=requests.exceptions.ConnectionError("connection refused"),
        ):
            with self.assertRaisesRegex(
                TrackApiError,
                "Track API request failed for genome 'genome-id': connection refused",
            ):
                self.client().get_track_categories("genome-id")
