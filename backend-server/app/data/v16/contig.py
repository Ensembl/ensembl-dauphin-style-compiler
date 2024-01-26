import random
from .dataalgorithm import data_algorithm
from command.coremodel import DataHandler, Panel, DataAccessor
from command.response import Response
from model.bigbed import get_bigbed

DOMINO_COUNT = 200
"""
Shimmering is the process of making sure we give similar weight to two senses present in a track when so far out
that they occupy less than one pixel. To achieve this, the panel is divided into a large number of equal-sized 
"dominoes", the number chosen such that they occupy at least a couple of pixels, but not much more. The base-pairs
corresponding to each domino are examined for presence and absence throughout. If both are present, a domino is
chosen with half set and half not set (which half is random). If only one is present, the whole domino is that
colour. Without applying dominoing either one colour dominates over the other misleadingly or the track degenerates
into a misleadingly uniform intermediate colour, hiding the variety of densities within.

Attributes:
    DOMINO_COUNT (int): we give similar weight to two senses present in a track when so far out
    that they occupy less than one pixel. To achieve this, the panel is divided into a large number of equal-sized 
"dominoes", the number chosen such that they occupy at least a couple of pixels, but not much more

"""


def shimmer_push(
        out_positions: list[list[int]], out_sense: list[bool], start: int, end: int, sense: bool
    ) -> None:
    """

    Args:
        out_positions (list[list[int]]):
        out_sense (list[bool]):
        start (int):
        end (int):
        sense (bool):

    Returns:
        None
    """
    if len(out_sense) > 0 and sense == out_sense[-1] and out_positions[-1][1] == start:
        out_positions[-1][1] = int(end)
    else:
        out_positions.append([int(start), int(end)])
        out_sense.append(sense)


def shimmer(
        positions: list[tuple[int]], sense: list[bool], start: int, end: int
    ) -> tuple[list[tuple[int, int]], list[int]]:
    """

    Args:
        positions (list[tuple[int]]):
        sense (list[bool]):
        start (int):
        end (int):

    Returns:
        tuple

    """
    domino_bp = (end - start) / DOMINO_COUNT
    domino_onoff = [0] * DOMINO_COUNT
    for ((start_p, end_p), sense) in zip(positions, sense):
        start_d = int((start_p - start) / domino_bp)
        end_d = int((end_p - start) / domino_bp)
        for domino in range(start_d, min(end_d,len(domino_onoff))):
            domino_onoff[domino] |= (2 if sense else 1)
    out_position = []
    out_sense = []
    for domino in range(0, DOMINO_COUNT):
        start_d = start + domino * domino_bp
        end_d = start_d + domino_bp
        if domino_onoff[domino] == 3:
            flip = random.randint(0, 1) != 0
            shimmer_push(out_position, out_sense, start_d, (start_d + end_d) / 2.0, flip)
            shimmer_push(out_position, out_sense, (start_d + end_d) / 2.0, end_d, not flip)
        elif domino_onoff[domino] == 2:
            shimmer_push(out_position, out_sense, start_d, end_d, True)
        elif domino_onoff[domino] == 1:
            shimmer_push(out_position, out_sense, start_d, end_d, False)
    return out_position, out_sense


def get_contig(data_accessor: DataAccessor, panel: Panel, do_shimmer: bool) -> dict:
    item = panel.get_chrom(data_accessor).item_path("contigs")
    data = get_bigbed(data_accessor, item, panel.start, panel.end)
    positions = []
    senses = []
    for line in data:
        (contig_start, contig_end, rest) = line
        (name, value, sense) = rest.split("\t")
        positions.append((contig_start, contig_end))
        senses.append(sense == '+')
    if do_shimmer:
        (positions, senses) = shimmer(positions, senses, panel.start, panel.end)
    return {
        "sense": data_algorithm("NRL",senses),
        'contig_starts': data_algorithm("NDZRL",[x[0] for x in positions]),
        'contig_lengths': data_algorithm("NDZRL",[x[1] - x[0] for x in positions]),
    }

class ContigDataHandler16(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope: dict, accept: str) -> dict:
        """

        Args:
            data_accessor (object):
            panel (object):
            scope (object):

        Returns:

        """
        return get_contig(data_accessor, panel, False)

class ShimmerContigDataHandler16(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope: dict, accept: str) -> dict:
        """

        Args:
            data_accessor (object):
            panel (object):
            scope (object):

        Returns:

        """
        return get_contig(data_accessor, panel, True)
