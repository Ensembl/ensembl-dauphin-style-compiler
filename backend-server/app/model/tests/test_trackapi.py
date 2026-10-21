import unittest
from unittest.mock import Mock, patch

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

    def test_configuration_request_pins_dataset_and_checks_scope(self):
        payload = {"genome_id": "genome", "dataset_id": "dataset", "selections": []}
        with patch("model.trackapi.requests.get", return_value=Mock(json=Mock(return_value=payload))) as get:
            self.assertEqual(self.client().get_transcriptomic_configuration("genome", "dataset"), payload)
            get.assert_called_once_with(
                "https://track-api.example/configuration/genome/transcriptomic",
                params={"dataset_id": "dataset"}, timeout=5,
            )
            for wrong_scope in ({**payload, "genome_id": "other"}, {**payload, "dataset_id": "other"}):
                get.return_value.json.return_value = wrong_scope
                with self.assertRaisesRegex(TrackApiError, "does not match"):
                    self.client().get_transcriptomic_configuration("genome", "dataset")

    def test_configuration_failures_are_normalized(self):
        for error in (requests.exceptions.Timeout("timeout"), ValueError("invalid JSON")):
            with patch("model.trackapi.requests.get", side_effect=error):
                with self.assertRaisesRegex(TrackApiError, "configuration request failed"):
                    self.client().get_transcriptomic_configuration("genome", "dataset")

    def test_bulk_metadata_is_requested_in_batches_of_at_most_100(self):
        ids = [f"track-{index}" for index in range(201)]
        batches = []

        def response(_url, params, timeout):
            batch = params["track_ids"].split(",")
            batches.append(batch)
            return Mock(json=Mock(return_value={"tracks": [{"track_id": track_id} for track_id in batch]}))

        with patch("model.trackapi.requests.get", side_effect=response):
            records = self.client().get_tracks(ids)
        self.assertEqual([len(batch) for batch in batches], [100, 100, 1])
        self.assertEqual([record["track_id"] for record in records], ids)

    def test_incomplete_or_duplicate_bulk_results_are_rejected(self):
        for records in ([{"track_id": "first"}], [{"track_id": "first"}] * 2):
            with patch("model.trackapi.requests.get", return_value=Mock(json=Mock(return_value={"tracks": records}))):
                with self.assertRaisesRegex(TrackApiError, "does not match"):
                    self.client().get_tracks(["first", "second"])

    def test_bulk_request_transport_error_is_normalized(self):
        with patch("model.trackapi.requests.get", side_effect=requests.exceptions.Timeout("timeout")):
            with self.assertRaisesRegex(TrackApiError, "bulk request failed"):
                self.client().get_tracks(["track"])
