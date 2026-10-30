import unittest

from data.v16.contig import shimmer


class ShimmerBoundaryTests(unittest.TestCase):
    def test_contigs_only_cover_their_overlap_with_the_panel(self):
        # Include the crashing case and smaller negative indices that previously
        # wrapped around and coloured the wrong end of the panel.
        for contig, overlap in (
            ((0, 8192), (4096, 5120)),
            ((3584, 4608), (4096, 4608)),
            ((4608, 8192), (4608, 5120)),
            ((4352, 4864), (4352, 4864)),
            ((4096, 5120), (4096, 5120)),
            ((0, 4096), None),
            ((5120, 8192), None),
            ((4608, 4608), None),
        ):
            for sense in (True, False):
                with self.subTest(contig=contig, sense=sense):
                    positions, senses = shimmer([contig], [sense], 4096, 5120)
                    self.assertEqual(senses, [sense] * len(positions))
                    if overlap is None:
                        self.assertEqual(positions, [])
                    else:
                        self.assertEqual((positions[0][0], positions[-1][1]), overlap)
                        self.assertEqual(sum(end - start for start, end in positions), overlap[1] - overlap[0])
