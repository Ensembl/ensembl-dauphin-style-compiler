import copy
import unittest
from contextlib import ExitStack
from unittest.mock import patch

import cbor2
from fastapi import FastAPI
from starlette.testclient import TestClient

from api.resources.data import router


DATA_PAYLOAD = [
    ["", ""], "gene", ["genome:1", 16, 0],
    {"datafile": ["tracks/gene.bb"], "expanded": []}, "release",
]
CLIENT_METRICS = {
    "type": "Client",
    "datastream": {"names": ["gene"], "keys": ["data"], "datapoints": [[0, 0, 16, 0, 1, 128]]},
    "programrun": {"names": ["gene"], "datapoints": [[0, 16, True, 1.25, 2.5]]},
    "general": [["render", ["genome"], ["genome-id"], ["duration"], [[[0], [0], [0], [1.5]]]]],
}
COMMANDS = (
    (0, None), (1, ["ensembl-webteam/core", "gene", 1]), (2, ["genome:1"]),
    (4, DATA_PAYLOAD), (5, ["focus:gene:genome:gene-id"]), (6, CLIENT_METRICS),
    (7, ["general", "track-id"]), (8, ["namespace", "column"]),
)


class DataPacketValidationTests(unittest.TestCase):
    def setUp(self):
        app = FastAPI()
        app.include_router(router, prefix="/api/data")
        self.client = TestClient(app, raise_server_exceptions=False)
        self.packet = {"channel": ["", ""], "requests": [[0, 0, None]], "version": {"egs": 16}}
        patches = ExitStack()
        self.addCleanup(patches.close)
        self.process = patches.enter_context(patch("api.resources.data.process_packet"))
        self.warning = patches.enter_context(patch("api.resources.data.logging.warning"))

    def post(self, packet=None, *, body=None, priority="hi", **kwargs):
        self.process.reset_mock()
        self.warning.reset_mock()
        if body is None:
            body = cbor2.dumps(self.packet if packet is None else packet)
        return self.client.post(f"/api/data/{priority}?stamp=test-stamp", data=body, **kwargs)

    def assert_rejected(self, packet=None, *, body=None):
        response = self.post(packet, body=body)
        self.assertEqual(response.status_code, 400)
        self.assertIn("detail", response.json())
        self.process.assert_not_called()
        return response

    def test_captured_production_body_is_rejected(self):
        body = bytes.fromhex("676368616e6e656c606068726571756573747301007273696f6e6365677310")
        self.assert_rejected(body=body)

    def test_decoder_failures_and_trailing_data_are_rejected(self):
        bodies = [b"", b"\x1c", cbor2.dumps(self.packet)[:-1], b"\x81" * 2000 + b"\xf6",
                  cbor2.dumps(self.packet) + b"garbage"]
        bodies.extend(cbor2.dumps(cbor2.CBORTag(tag, value)) for tag, value in (
            (35, None), (4, ["invalid", 1]), (4, [0, "invalid"]),
        ))
        for body in bodies:
            with self.subTest(body=body[:40]):
                self.assert_rejected(body=body)

    def test_packet_envelope_is_validated(self):
        for value in ("channel", [], None, {}, {"channel": ["", ""]}, {"requests": []}):
            with self.subTest(value=value):
                self.assert_rejected(body=cbor2.dumps(value))
        for field, value in (
            ("channel", []), ("channel", [0, ""]), ("channel", ""),
            ("requests", {}), ("requests", [[0, 0]]), ("requests", [[True, 0, None]]),
            ("requests", [[0, [], None]]), ("requests", [[-1, 0, None]]),
            ("version", "16"), ("version", {"egs": "16"}), ("version", {"egs": True}),
        ):
            with self.subTest(field=field, value=value):
                self.assert_rejected(dict(self.packet, **{field: value}))

    def test_commands_validate_payload_shape(self):
        cases = [(command, None) for command, _ in COMMANDS if command != 0]
        cases += [
            (0, []), (1, ["group", "name", "1"]), (1, [[], "name", 1]),
            (1, ["group", "name", 2**32]), (2, ["a", "b"]), (2, [[]]),
            (4, DATA_PAYLOAD[:-1]), (5, [None]), (6, {"type": []}),
            (6, {"type": "Client"}), (7, ["general", []]), (8, ["namespace"]),
        ]
        for command, payload in cases:
            with self.subTest(command=command, payload=payload):
                self.assert_rejected(dict(self.packet, requests=[[0, command, payload]]))

    def test_data_command_validates_nested_fields(self):
        for field, value in (
            (0, None), (1, []), (2, []), (2, ["genome:1", "16", 0]),
            (2, ["genome:1", -1, 0]), (2, ["genome:1", 64, 0]),
            (2, ["genome:1", 16, False]), (2, ["genome:1", 16, 2**64]),
            (3, None), (3, {"datafile": "gene.bb"}), (3, {"datafile": []}),
            (3, {"genome": []}), (3, {"expanded": [1]}), (4, None),
        ):
            with self.subTest(field=field, value=value):
                payload = copy.deepcopy(DATA_PAYLOAD)
                payload[field] = value
                self.assert_rejected(dict(self.packet, requests=[[0, 4, payload]]))

    def test_client_metrics_validate_tables_indices_and_values(self):
        for path, value in (
            (("datastream",), None), (("datastream", "names"), [1]),
            (("datastream", "datapoints"), [[0, 0]]),
            (("datastream", "datapoints", 0, 0), 1),
            (("datastream", "datapoints", 0, 1), -1),
            (("datastream", "datapoints", 0, 3), 2),
            (("datastream", "datapoints", 0, 5), 2**64),
            (("programrun",), {}), (("programrun", "datapoints"), [[0]]),
            (("programrun", "datapoints", 0, 0), 1),
            (("programrun", "datapoints", 0, 2), 1),
            (("programrun", "datapoints", 0, 3), "slow"),
            (("general",), None), (("general", 0), ["render"]),
            (("general", 0, 1), [1]), (("general", 0, 4, 0, 0), [1]),
            (("general", 0, 4, 0, 1), [1]), (("general", 0, 4, 0, 2), [1]),
            (("general", 0, 4, 0, 1), []), (("general", 0, 4, 0, 3), []),
            (("general", 0, 4, 0, 3), [float("nan")]),
        ):
            with self.subTest(path=path, value=value):
                payload = copy.deepcopy(CLIENT_METRICS)
                target = payload
                for key in path[:-1]:
                    target = target[key]
                target[path[-1]] = value
                self.assert_rejected(dict(self.packet, requests=[[0, 6, payload]]))

    def test_invalid_later_command_rejects_batch_before_dispatch(self):
        packet = dict(self.packet, requests=[[0, 0, None], [1, 4, None]])
        response = self.assert_rejected(packet)
        self.assertEqual(response.json(), {"detail": "Invalid payload for command 4"})

    def test_valid_commands_and_legacy_packets_keep_cbor_responses(self):
        commands = COMMANDS + (
            (4, ["self()", "gene", ["genome:1", 0, 0], {}, "dump"]),
            (6, {"type": "Error", "text": "example"}), (6, {"type": "custom-metric"}),
            (255, {"example": "unsupported command"}),
        )
        packets = [
            dict(self.packet, requests=[[i, command, payload] for i, (command, payload) in enumerate(commands)]),
            dict(self.packet, channel="self()", version=None),
            {"channel": ["ensembl", "main"], "requests": []},
            dict(self.packet, version={"egs": 999}),
        ]
        self.process.return_value = ([[0, cbor2.dumps([0, {}])]], [], [], ["", ""], [])
        for packet in packets:
            for priority in ("hi", "lo"):
                with self.subTest(packet=packet, priority=priority):
                    response = self.post(packet, priority=priority)
                    self.assertEqual(response.status_code, 200)
                    self.assertEqual(response.headers["content-type"], "application/cbor")
                    self.assertEqual(cbor2.loads(response.content), {
                        "responses": [[0, [0, {}]]], "programs": [], "eardos": [],
                        "channel": ["", ""], "tracks-packed": [],
                    })
                    self.process.assert_called_once_with(packet, priority == "hi")

    def test_rejection_log_preserves_decoder_error_and_correlation_without_body(self):
        body = b"\xa1gprivate"
        with self.assertRaises(cbor2.CBORDecodeError) as original:
            cbor2.loads(body)
        response = self.post(body=body, headers={
            "x-request-id": "test-id", "referer": "https://example.org/browser",
        })
        self.assertEqual(response.status_code, 400)
        self.warning.assert_called_once()
        message = self.warning.call_args.args[0] % self.warning.call_args.args[1:]
        for value in (type(original.exception).__name__, repr(str(original.exception)),
                      "test-stamp", "test-id", "https://example.org/browser"):
            self.assertIn(value, message)
        self.assertNotIn("private", message)

    def test_unexpected_decoder_and_backend_errors_remain_server_errors(self):
        for target, error in (("cbor2.load", RuntimeError), ("cbor2.load", MemoryError),
                              ("process_packet", TypeError)):
            with self.subTest(target=target, error=error):
                with patch(f"api.resources.data.{target}", side_effect=error("server failure")):
                    response = self.post()
                self.assertEqual(response.status_code, 500)
                self.warning.assert_not_called()


if __name__ == "__main__":
    unittest.main()
