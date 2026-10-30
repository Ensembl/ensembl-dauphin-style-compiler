import json
import unittest
from types import SimpleNamespace
from unittest.mock import Mock, patch

import cbor2
import requests
from fastapi import FastAPI
from starlette.testclient import TestClient

from api.resources.data import router
from command.packet import do_request_remote
from core.exceptions import UpstreamRequestError
from model.datalocator import AccessItem, MetadataApiClient, RefgetAccessMethod, UrlAccessMethod
from model.graphql import CoreApiClient
from model.trackapi import TrackApiClient, TrackDatafileResolver
from model.upstream import request_json, request_upstream
from model.version import Version


def response(status, text, url="https://upstream.example"):
    result = requests.Response()
    result.status_code, result.url = status, url
    result._content = text.encode("utf-8") if isinstance(text, str) else text
    return result


class UpstreamFailureTests(unittest.TestCase):
    def setUp(self):
        self.refget = RefgetAccessMethod("https://refget.example/sequence", AccessItem("seq", "genome", "a" * 32))
        self.track = TrackApiClient.__new__(TrackApiClient)
        self.track._host = "https://track.example"
        self.core = CoreApiClient.__new__(CoreApiClient)
        self.core._core_api_url = "https://core.example/graphql"

    def test_http_clients_preserve_failed_request_details(self):
        metadata = MetadataApiClient("https://metadata.example")
        source = UrlAccessMethod("https://data.example", AccessItem.track_file("tracks/gene.bb", "genome"))
        cases = (
            ("Refget", "GET", self.refget.url + "?start=512&end=768", lambda: self.refget.get(512, 256)),
            ("Refget", "GET", self.refget.url + "/metadata", self.refget.get_length),
            ("Track API", "GET", "https://track.example/track/id", lambda: self.track.get_track("id")),
            ("Track API", "GET", "https://track.example/track_categories/genome", lambda: self.track.get_track_categories("genome")),
            ("Metadata API", "GET", "https://metadata.example/genome/genome/checksum/1", lambda: metadata.get_checksum("genome", "1")),
            ("Core API", "POST", self.core._core_api_url, lambda: self.core.get_gene_location(("genome", "gene"))),
            ("Data source", "GET", source.url, source.get),
            ("Genome Browser", "POST", "https://delegate.example/hi", lambda: do_request_remote("https://delegate.example", ["", ""], [], True, Version({"egs": 16}))),
        )
        for service, method, url, call in cases:
            with self.subTest(service=service, url=url):
                with patch(f"model.upstream.requests.{method.lower()}", return_value=response(400, "Invalid requested range", url)):
                    with self.assertRaises(UpstreamRequestError) as caught:
                        call()
                error = caught.exception
                self.assertEqual((error.service, error.method, error.url), (service, method, url))
                expected_status = 502 if service in ("Core API", "Data source") or url.endswith("/metadata") else 400
                self.assertEqual((error.upstream_status, error.status_code, error.detail), (400, expected_status, "Invalid requested range"))

    def test_dependency_failures_keep_server_error_statuses(self):
        for failure, status in (
            (response(503, "Unavailable"), 502),
            (response(403, "Service credentials rejected"), 502),
            (requests.Timeout("timed out"), 504),
            (requests.ConnectionError("connection refused"), 502),
        ):
            with self.subTest(failure=failure):
                options = {"side_effect": failure} if isinstance(failure, Exception) else {"return_value": failure}
                with patch("model.upstream.requests.get", **options):
                    with self.assertRaises(UpstreamRequestError) as caught:
                        request_upstream("Refget", "GET", self.refget.url)
                self.assertEqual(caught.exception.status_code, status)

    def test_invalid_upstream_responses_are_bad_gateway_errors(self):
        cases = (
            (lambda: request_json("Example", "GET", "https://upstream.example"), "not JSON"),
            (lambda: self.track.get_track("id"), "[]"),
            (lambda: self.track.get_track_categories("genome"), "{}"),
            (self.refget.get_length, '{"metadata":{"length":-1}}'),
        )
        for call, text in cases:
            with self.subTest(text=text):
                with patch("model.upstream.requests.get", return_value=response(200, text)):
                    with self.assertRaises(UpstreamRequestError) as caught:
                        call()
                self.assertEqual(caught.exception.status_code, 502)
        with patch("model.upstream.requests.post", return_value=response(200, b"invalid CBOR")):
            with self.assertRaises(UpstreamRequestError) as caught:
                do_request_remote("https://delegate.example", ["", ""], [], True, Version({"egs": 16}))
        self.assertEqual(caught.exception.status_code, 502)

    def test_successful_reads_are_preserved(self):
        for call, body, expected in (
            (lambda: self.refget.get(512, 256), "ACGT", b"ACGT"),
            (self.refget.get_length, '{"metadata":{"length":400}}', 400),
            (lambda: self.track.get_track("id"), '{"track_id":"id"}', {"track_id": "id"}),
            (lambda: self.track.get_track_categories("genome"), '{"track_categories":[]}', []),
        ):
            with self.subTest(body=body):
                with patch("model.upstream.requests.get", return_value=response(200, body)):
                    self.assertEqual(call(), expected)
        remote = {"responses": []}
        with patch("model.upstream.requests.post", return_value=response(200, cbor2.dumps(remote))):
            self.assertEqual(do_request_remote("https://delegate.example", ["", ""], [], True, Version({"egs": 16})), remote)

    def test_graphql_errors_propagate_and_successful_locations_are_preserved(self):
        for body in ({"errors": [{"message": "Lookup failed"}]}, []):
            with patch("model.upstream.requests.post", return_value=response(200, json.dumps(body))):
                with self.assertRaises(UpstreamRequestError) as caught:
                    self.core.get_gene_location(("genome", "gene"))
            self.assertEqual(caught.exception.status_code, 502)
        body = {"data": {"gene": {"slice": {"region": {"name": "1"}, "location": {"start": 100, "end": 200}}}}}
        with patch("model.upstream.requests.post", return_value=response(200, json.dumps(body))):
            self.assertEqual(self.core.get_gene_location(("genome", "gene")), ("genome:1", 50, 250))

    def test_data_upstream_failures_return_correlated_http_errors_without_tracebacks(self):
        app = FastAPI()
        app.include_router(router, prefix="/api/data")
        client = TestClient(app)
        chromosome = SimpleNamespace(item_seq_path=lambda _: AccessItem("seq", "genome", "a" * 32))
        accessor = SimpleNamespace(
            supported_versions=[16], cache=Mock(get_data=Mock(return_value=None)),
            data_model=SimpleNamespace(stick=lambda _: chromosome),
            resolver=SimpleNamespace(find_override=lambda _: None, get=lambda _: self.refget),
            track_datafiles=TrackDatafileResolver(),
        )
        accessor.track_datafiles._client = self.track
        packet = {"channel": ["", ""], "version": {"egs": 16}, "requests": [
            [0, 4, [["", ""], "zoomed-seq", ["genome:1", 8, 2], {"datafile": ["tracks/seq"]}, "release"]],
        ]}
        detail = '{"detail":"Requested start is beyond end of sequence"}'
        for service, endpoint, scope, url in (
            ("Refget", "zoomed-seq", {"datafile": ["tracks/seq"]}, self.refget.url + "?start=512&end=768"),
            ("Track API", "gene", {"genome": ["genome"]}, self.track._host + "/track_categories/genome"),
        ):
            packet["requests"][0][2][1], packet["requests"][0][2][3] = endpoint, scope
            for failure, status in ((response(400, detail, url), 400), (response(503, "Unavailable", url), 502), (requests.Timeout("timed out"), 504)):
                with self.subTest(service=service, status=status):
                    options = {"side_effect": failure} if isinstance(failure, Exception) else {"return_value": failure}
                    # Real packet dispatch and sequence handlers; only dependencies are replaced.
                    with patch("command.packet.data_accessor_collection.get", return_value=accessor), \
                         patch("model.upstream.requests.get", **options) as get, \
                         patch("api.resources.data.logging.warning") as warning:
                        result = client.post("/api/data/hi?stamp=test-stamp", data=cbor2.dumps(packet), headers={
                            "referer": "https://www.ensembl.org/genome-browser/genome?location=1:512-768",
                            "x-request-id": "test-id",
                        })
                    self.assertEqual(result.status_code, status)
                    self.assertEqual(result.json(), {"detail": f"{service} request failed"})
                    warning.assert_called_once()
                    message = warning.call_args.args[0] % warning.call_args.args[1:]
                    for value in ("test-stamp", "test-id", "genome-browser/genome?location=1:512-768", url, "incoming_url=", "upstream_status="):
                        self.assertIn(value, message)
                    self.assertIn(detail if status == 400 else "Unavailable" if status == 502 else "timed out", message)
                    self.assertNotIn("exc_info", warning.call_args.kwargs)
                    get.assert_called_once_with(url, timeout=5)
                    accessor.cache.store_data.assert_not_called()
