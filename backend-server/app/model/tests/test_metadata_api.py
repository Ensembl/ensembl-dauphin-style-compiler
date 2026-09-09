import unittest
from unittest.mock import patch

from core.exceptions import RequestException
from model.datalocator import AccessItem, FileDataSource, MetadataAccessMethod, MetadataApiClient, S3DataSource
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


class MetadataApiClientTests(unittest.TestCase):
    def test_karyotype_is_cached_locally_and_in_shared_cache(self):
        cache = FakeCache()
        first = MetadataApiClient("https://metadata.example/api/metadata", cache)
        response = FakeResponse(200, payload=[{"name": "1", "length": 10}])

        with patch("model.datalocator.requests.get", return_value=response) as get:
            self.assertEqual(first.get_karyotype("genome"), {"1": 10})
            self.assertEqual(first.get_karyotype("genome"), {"1": 10})
            self.assertEqual(get.call_count, 1)

        second = MetadataApiClient("https://metadata.example/api/metadata", cache)
        with patch("model.datalocator.requests.get") as get:
            self.assertEqual(second.get_karyotype("genome"), {"1": 10})
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

    def test_invalid_karyotype_is_not_cached(self):
        client = MetadataApiClient("https://metadata.example/api/metadata", FakeCache())

        with patch(
            "model.datalocator.requests.get",
            return_value=FakeResponse(200, payload=[{"name": "1", "length": 0}]),
        ):
            with self.assertRaises(RequestException):
                client.get_karyotype("genome")

        self.assertNotIn("genome", client._karyotypes)


class SpeciesKaryotypeTests(unittest.TestCase):
    def test_multiple_chromosomes_share_one_karyotype_lookup(self):
        class Resolver:
            def __init__(self):
                self.karyotype_calls = 0

            def get(self, item):
                if item.variety == "chrom-hashes":
                    return type("Checksum", (), {"get_checksum": lambda _self: "a" * 32})()
                self.karyotype_calls += 1
                return type("Karyotype", (), {"get_karyotype": lambda _self: {"1": 10, "2": 20}})()

        resolver = Resolver()
        accessor = type("Accessor", (), {"resolver": resolver})()
        species = Species("59871324-7803-4234-856e-2a2bd96d7b3c")

        self.assertEqual(species.chromosome(accessor, f"{species.genome_id}:1").size, 10)
        self.assertEqual(species.chromosome(accessor, f"{species.genome_id}:2").size, 20)
        self.assertEqual(resolver.karyotype_calls, 1)


class MetadataDatasourceRoutingTests(unittest.TestCase):
    def test_file_and_s3_sources_route_both_chromosome_metadata_varieties(self):
        sources = [
            FileDataSource({"root": "/data", "metadata_url": "https://metadata.example"}),
            S3DataSource({"url": "https://data.example", "metadata_url": "https://metadata.example"}),
        ]

        for source in sources:
            for variety in ("chrom-hashes", "chrom-sizes"):
                with self.subTest(source=type(source).__name__, variety=variety):
                    method = source.resolve(AccessItem(variety, "genome", "1"))
                    self.assertIsInstance(method, MetadataAccessMethod)
