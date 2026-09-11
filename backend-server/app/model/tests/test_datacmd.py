import unittest

from command.datacmd import DataHandler


class StaticDatafileResolutionTests(unittest.TestCase):
    def setUp(self):
        class TrackDatafiles:
            @staticmethod
            def datafile_for_endpoint(genome_id, endpoint):
                return f"tracks/{genome_id}/{endpoint}.bb"

        class DataAccessor:
            track_datafiles = TrackDatafiles()

        class Panel:
            @staticmethod
            def get_chrom(_data_accessor):
                class Chromosome:
                    genome_id = "default-genome"

                return Chromosome()

        self.data_accessor = DataAccessor()
        self.panel = Panel()

    def test_focus_request_without_datafile_uses_track_api_path(self):
        scope = {"genome": ["focus-genome"]}

        resolved = DataHandler._add_static_datafile(
            self.data_accessor, self.panel, "variant-summary", scope
        )

        self.assertEqual(
            resolved["datafile"], ["tracks/focus-genome/variant-summary.bb"]
        )

    def test_dynamic_request_keeps_its_explicit_datafile(self):
        scope = {"datafile": ["tracks/uuid/variant-summary.bw"]}

        resolved = DataHandler._add_static_datafile(
            self.data_accessor, self.panel, "variant-summary", scope
        )

        self.assertIs(resolved, scope)


if __name__ == "__main__":
    unittest.main()