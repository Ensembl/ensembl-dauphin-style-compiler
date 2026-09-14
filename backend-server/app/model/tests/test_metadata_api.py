import unittest
from unittest.mock import patch
import requests

from core.exceptions import RequestException
from model.datalocator import (
    AccessItem,
    FileDataSource,
    MetadataAccessMethod,
    MetadataApiClient,
    RefgetAccessMethod,
    S3DataSource,
)
from model.species import Species


class FakeCache:
    def __init__(self):
        self.values = {}

    def get_metadata(self, key):
        return self.values.get(tuple(key))

    def set_metadata(self, key, value):
        self.values[tuple(key)] = value


class FakeResponse:
    def __init__(self, status_code, text="", payload=None):
        self.status_code = status_code
        self.text = text
        self._payload = payload

    def json(self):
        if isinstance(self._payload, Exception):
            raise self._payload
        return self._payload


class RefgetMetadataTests(unittest.TestCase):
    def test_length_is_fetched_from_metadata_and_cached_in_shared_cache(self):
        cache = FakeCache()
        first = RefgetAccessMethod("https://refget.example/sequence", AccessItem("chrom-sizes", "genome", "a" * 32), cache)
        response = FakeResponse(200, payload={"metadata": {"length": 10}})

        with patch("model.datalocator.requests.get", return_value=response) as get:
            self.assertEqual(first.get_length(), 10)
            get.assert_called_once_with(f"https://refget.example/sequence/{'a' * 32}/metadata", timeout=5)

        second = RefgetAccessMethod("https://refget.example/sequence", AccessItem("chrom-sizes", "genome", "a" * 32), cache)
        with patch("model.datalocator.requests.get") as get:
            self.assertEqual(second.get_length(), 10)
            get.assert_not_called()

    def test_checksum_404_is_cached_as_an_authoritative_miss(self):
        cache = FakeCache()
        first = MetadataApiClient("https://metadata.example/api/metadata", cache)

        with patch("model.datalocator.requests.get", return_value=FakeResponse(404)) as get:
            self.assertIsNone(first.get_checksum("genome", "missing"))
            self.assertIsNone(first.get_checksum("genome", "missing"))
            self.assertEqual(get.call_count, 1)

        second = MetadataApiClient("https://metadata.example/api/metadata", cache)
        with patch("model.datalocator.requests.get") as get:
            self.assertIsNone(second.get_checksum("genome", "missing"))
            get.assert_not_called()

    def test_invalid_refget_lengths_are_not_cached(self):
        cache = FakeCache()
        client = RefgetAccessMethod("https://refget.example/sequence", AccessItem("chrom-sizes", "genome", "a" * 32), cache)

        invalid_payloads = [
            None,
            {},
            {"metadata": {}},
            {"metadata": {"length": True}},
            {"metadata": {"length": 0}},
            {"metadata": {"length": -1}},
            {"metadata": {"length": "10"}},
        ]
        for payload in invalid_payloads:
            with self.subTest(payload=payload):
                with patch("model.datalocator.requests.get", return_value=FakeResponse(200, payload=payload)):
                    with self.assertRaises(RequestException):
                        client.get_length()

        self.assertEqual(cache.values, {})

    def test_refget_metadata_errors_raise_request_exception(self):
        client = RefgetAccessMethod("https://refget.example/sequence", AccessItem("chrom-sizes", "genome", "a" * 32))

        with patch("model.datalocator.requests.get", side_effect=requests.Timeout("timed out")):
            with self.assertRaises(RequestException):
                client.get_length()
        with patch("model.datalocator.requests.get", return_value=FakeResponse(404)):
            with self.assertRaises(RequestException):
                client.get_length()
        with patch("model.datalocator.requests.get", return_value=FakeResponse(200, payload=ValueError())):
            with self.assertRaises(RequestException):
                client.get_length()


class SpeciesRefgetMetadataTests(unittest.TestCase):
    def test_chromosome_size_uses_the_resolved_checksum(self):
        class Resolver:
            def __init__(self):
                self.size_checksums = []

            def get(self, item):
                if item.variety == "chrom-hashes":
                    return type("Checksum", (), {"get_checksum": lambda _self: "a" * 32})()
                self.size_checksums.append(item.chromosome)
                return type("Metadata", (), {"get_length": lambda _self: 10})()

        resolver = Resolver()
        accessor = type("Accessor", (), {"resolver": resolver})()
        species = Species("59871324-7803-4234-856e-2a2bd96d7b3c")

        self.assertEqual(species.chromosome(accessor, f"{species.genome_id}:1").size, 10)
        self.assertEqual(resolver.size_checksums, ["a" * 32])


class MetadataDatasourceRoutingTests(unittest.TestCase):
    def test_file_and_s3_sources_route_chromosome_names_to_metadata_and_checksums_to_refget(self):
        sources = [
            FileDataSource({"root": "/data", "metadata_url": "https://metadata.example", "refget_url": "https://refget.example/sequence"}),
            S3DataSource({"url": "https://data.example", "metadata_url": "https://metadata.example", "refget_url": "https://refget.example/sequence"}),
        ]

        for source in sources:
            with self.subTest(source=type(source).__name__, variety="chrom-hashes"):
                method = source.resolve(AccessItem("chrom-hashes", "genome", "1"))
                self.assertIsInstance(method, MetadataAccessMethod)
            with self.subTest(source=type(source).__name__, variety="chrom-sizes"):
                method = source.resolve(AccessItem("chrom-sizes", "genome", "a" * 32))
                self.assertIsInstance(method, RefgetAccessMethod)
