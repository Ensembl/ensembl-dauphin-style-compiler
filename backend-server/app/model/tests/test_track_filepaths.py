import unittest

from model.datalocator import (
    AccessItem,
    FileAccessMethod,
    TrackFilepathError,
    UrlAccessMethod,
    validate_track_filepath,
)


class TrackFilepathTests(unittest.TestCase):
    def test_track_filepath_is_used_verbatim_below_file_root(self):
        item = AccessItem.track_file(
            "tracks/59871324-7803-4234-856e-2a2bd96d7b3c/variants.bb",
            genome="ignored-genome",
            chromosome="1",
        )

        self.assertEqual(
            FileAccessMethod("/data/browser", item).file,
            "/data/browser/tracks/59871324-7803-4234-856e-2a2bd96d7b3c/variants.bb",
        )
        self.assertEqual(
            UrlAccessMethod("https://data.example/browser", item).url,
            "https://data.example/browser/tracks/59871324-7803-4234-856e-2a2bd96d7b3c/variants.bb",
        )

    def test_legacy_infrastructure_items_keep_genome_relative_paths(self):
        item = AccessItem("chrom-sizes", genome="genome-uuid")
        self.assertEqual(item.item_suffix(), "genome-uuid/chrom.sizes.ncd")

    def test_invalid_track_filepaths_are_rejected(self):
        for filepath in ("", "/data/track.bb", "../track.bb", "tracks/../track.bb", "tracks//track.bb", "tracks\\track.bb", "tracks/\x00track.bb"):
            with self.subTest(filepath=filepath):
                with self.assertRaises(TrackFilepathError):
                    validate_track_filepath(filepath)
