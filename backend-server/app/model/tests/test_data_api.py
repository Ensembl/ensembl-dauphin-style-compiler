import unittest
from unittest.mock import patch

import cbor2
from fastapi import FastAPI
from starlette.testclient import TestClient

from api.resources.data import router


class DataPacketValidationTests(unittest.TestCase):
    def setUp(self):
        app = FastAPI()
        app.include_router(router, prefix="/api/data")
        self.client = TestClient(app)
        self.packet = {
            "channel": ["", ""],
            "requests": [[0, 0, None]],
            "version": {"egs": 16},
        }

    def assert_rejected(self, body, detail):
        with patch("api.resources.data.process_packet") as process:
            with patch("api.resources.data.logging.warning"):
                response = self.client.post(
                    "/api/data/hi", data=body,
                    headers={"content-type": "application/cbor"},
                )
        self.assertEqual(response.status_code, 400)
        self.assertEqual(response.json(), {"detail": detail})
        process.assert_not_called()

    def test_captured_production_bodies_are_rejected_before_dispatch(self):
        for msgid in ("01", "02"):
            body = bytes.fromhex(
                "676368616e6e656c6060687265717565737473"
                + msgid + "007273696f6e6365677310"
            )
            with self.subTest(msgid=msgid):
                self.assertEqual(cbor2.loads(body), "channel")
                self.assert_rejected(body, "Expected a CBOR map")

    def test_invalid_and_truncated_cbor_are_rejected(self):
        for body in (b"", b"\xa1", b"\x1c", cbor2.dumps(self.packet)[:-1]):
            with self.subTest(body=body):
                self.assert_rejected(body, "Invalid CBOR body")

    def test_invalid_semantic_tags_are_rejected(self):
        for tag in (
            cbor2.CBORTag(35, None),
            cbor2.CBORTag(4, ["invalid", 1]),
            cbor2.CBORTag(4, [0, "invalid"]),
        ):
            with self.subTest(tag=tag):
                packet = dict(self.packet, requests=[[0, 0, tag]])
                self.assert_rejected(cbor2.dumps(packet), "Invalid CBOR body")

    def test_excessive_nesting_is_rejected(self):
        self.assert_rejected(b"\x81" * 2000 + b"\xf6", "Invalid CBOR body")

    def test_decoder_resource_errors_are_not_reclassified_as_bad_requests(self):
        with patch("api.resources.data.cbor2.load", side_effect=MemoryError("out of memory")):
            with self.assertRaisesRegex(MemoryError, "out of memory"):
                self.client.post("/api/data/hi", data=cbor2.dumps(self.packet))

    def test_unexpected_decoder_errors_return_server_errors(self):
        client = TestClient(self.client.app, raise_server_exceptions=False)
        for error_type in (RuntimeError, KeyError, MemoryError):
            with self.subTest(error_type=error_type):
                with patch("api.resources.data.cbor2.load", side_effect=error_type("decoder failure")):
                    with patch("api.resources.data.process_packet") as process:
                        with patch("api.resources.data.logging.warning") as warning:
                            response = client.post("/api/data/hi", data=cbor2.dumps(self.packet))
                self.assertEqual(response.status_code, 500)
                process.assert_not_called()
                warning.assert_not_called()

    def test_non_map_cbor_values_are_rejected(self):
        for value in ("channel", b"channel", [], 1, 1.5, True, None):
            with self.subTest(value=value):
                self.assert_rejected(cbor2.dumps(value), "Expected a CBOR map")

    def test_trailing_data_is_rejected(self):
        for suffix in (b"garbage", cbor2.dumps(self.packet)):
            with self.subTest(suffix=suffix):
                self.assert_rejected(
                    cbor2.dumps(self.packet) + suffix,
                    "Unexpected data after CBOR packet",
                )

    def test_required_fields_are_checked(self):
        for field in ("channel", "requests"):
            packet = dict(self.packet)
            del packet[field]
            with self.subTest(field=field):
                self.assert_rejected(cbor2.dumps(packet), "Missing channel or requests")

    def test_malformed_channels_are_rejected(self):
        for channel in (None, 0, True, {}, [], [""], ["", "", ""], [0, ""], "", "x"):
            with self.subTest(channel=channel):
                packet = dict(self.packet, channel=channel)
                self.assert_rejected(cbor2.dumps(packet), "Invalid channel")

    def test_requests_must_be_an_array(self):
        for requests in (None, "requests", {}, 1):
            with self.subTest(requests=requests):
                packet = dict(self.packet, requests=requests)
                self.assert_rejected(cbor2.dumps(packet), "Expected a requests array")

    def test_request_entries_must_have_an_id_type_and_payload(self):
        for request in (
            None, {}, "abc", [], [0, 0], [0, 0, None, None],
            ["0", 0, None], [0, [], None], [True, 0, None],
            [0, False, None], [-1, 0, None], [0, -1, None],
        ):
            with self.subTest(request=request):
                packet = dict(self.packet, requests=[request])
                self.assert_rejected(
                    cbor2.dumps(packet),
                    "Each request must contain a non-negative integer ID, command type and payload",
                )

    def test_malformed_versions_are_rejected(self):
        for version in ("16", 16, [], {"egs": "16"}, {"egs": []}, {"egs": True}, {"egs": -1}):
            with self.subTest(version=version):
                packet = dict(self.packet, version=version)
                self.assert_rejected(cbor2.dumps(packet), "Invalid version")

    def test_valid_packets_keep_cbor_responses_and_priority(self):
        packets = [
            self.packet,
            {"channel": ["", ""], "requests": []},
            dict(self.packet, channel=["ensembl", "main"]),
            dict(self.packet, channel="self()"),
            dict(self.packet, version=None),
            dict(self.packet, version={}),
            dict(self.packet, version={"egs": 999}),
            dict(self.packet, requests=[[1, 255, {"example": "payload"}]]),
        ]
        response_payload = [0, {"namespace": ["ensembl", "main"]}]
        result = (
            [[0, cbor2.dumps(response_payload)]], [], [], ["ensembl", "main"], [],
        )
        for packet in packets:
            for priority in ("hi", "lo"):
                with self.subTest(packet=packet, priority=priority):
                    with patch("api.resources.data.process_packet", return_value=result) as process:
                        response = self.client.post(
                            f"/api/data/{priority}", data=cbor2.dumps(packet),
                        )
                    self.assertEqual(response.status_code, 200)
                    self.assertEqual(response.headers["content-type"], "application/cbor")
                    self.assertEqual(cbor2.loads(response.content), {
                        "responses": [[0, response_payload]], "programs": [],
                        "eardos": [], "channel": ["ensembl", "main"], "tracks-packed": [],
                    })
                    process.assert_called_once_with(packet, priority == "hi")

    def test_rejection_log_has_correlation_metadata_without_payload(self):
        body = cbor2.dumps("malformed-payload-content")
        with patch("api.resources.data.logging.warning") as warning:
            response = self.client.post(
                "/api/data/hi?stamp=example-stamp", data=body,
                headers={"x-request-id": "example-id", "referer": "https://example.org/browser"},
            )
        self.assertEqual(response.status_code, 400)
        warning.assert_called_once()
        message = warning.call_args.args[0] % warning.call_args.args[1:]
        for value in ("example-stamp", "example-id", "https://example.org/browser"):
            self.assertIn(value, message)
        self.assertNotIn("malformed-payload-content", message)

    def test_rejection_log_preserves_decoder_exception_type_and_message(self):
        for body in (
            b"\xa1",
            cbor2.dumps(cbor2.CBORTag(35, None)),
            cbor2.dumps(cbor2.CBORTag(4, ["invalid", 1])),
            cbor2.dumps(cbor2.CBORTag(4, [0, "invalid"])),
        ):
            with self.subTest(body=body):
                with self.assertRaises(Exception) as original:
                    cbor2.loads(body)
                with patch("api.resources.data.logging.warning") as warning:
                    response = self.client.post("/api/data/hi", data=body)
                self.assertEqual(response.status_code, 400)
                self.assertEqual(response.json(), {"detail": "Invalid CBOR body"})
                warning.assert_called_once()
                message = warning.call_args.args[0] % warning.call_args.args[1:]
                self.assertIn("decoder_error_type=" + type(original.exception).__name__, message)
                self.assertIn("decoder_error=" + repr(str(original.exception)), message)

    def test_backend_errors_are_not_reclassified_as_bad_requests(self):
        for error_type in (TypeError, ValueError, ArithmeticError, RecursionError):
            with self.subTest(error_type=error_type):
                with patch("api.resources.data.process_packet", side_effect=error_type("backend bug")):
                    with self.assertRaisesRegex(error_type, "backend bug"):
                        self.client.post("/api/data/hi", data=cbor2.dumps(self.packet))


if __name__ == "__main__":
    unittest.main()
