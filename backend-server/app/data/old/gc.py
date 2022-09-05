import collections
import logging
from command.coremodel import DataHandler, Panel, DataAccessor
from command.response import Response
from model.bigbed import get_bigwig_stats, get_bigwig
from model.chromosome import Chromosome
from model.transcriptfile import TranscriptFileLine
from .numbers import delta, zigzag, lesqlite2, compress

SCALE = 4

"""
Attributes:
"""


def _get_gc(data_accessor: DataAccessor, chrom: Chromosome, panel: Panel) -> Response:
    """
    Use the DataAccessor provided to access the GC data for this chromosome and this panel.

    Args:
        data_accessor (DataAccessor): The means of accessing data
        chrom (Chromosome): The chromosome we want
        panel (Panel): The panel (ie genomic location, scale) we want

    Returns: A complete resonse object

    BUG: should return an out object unwrapped in a response so that this method could be
    used composed to other data.

    """
    item = chrom.item_path("gc")
    if panel.end - panel.start < 1000:
        (data, start, end) = get_bigwig(data_accessor, item, panel.start, panel.end)
    else:
        (data, start, end) = get_bigwig_stats(data_accessor, item, panel.start, panel.end)

    data = [1.0 if x is None else x for x in data]
    data = bytearray([round(x / SCALE) for x in data])
    out = {
        "values": compress(lesqlite2(zigzag(delta(data)))),
        "range": compress(lesqlite2([start, end]))
    }
    return Response(5, {'data': out})


class WiggleDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, _scope) -> Response:
        """
        Handle a request for wiggle data.

        Args:
            data_accessor (DataAccessor): The means of accessing data
            panel (Panel): The panel (ie genomic location, scale) we want
            scope (): extra scope info. Arg provided by caller but none defined for this endpoint.

        Returns: A complete Resonse object

        """
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        return _get_gc(data_accessor,chrom,panel)
