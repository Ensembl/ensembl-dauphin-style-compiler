import collections
import logging
from typing import Any, List
from command.coremodel import DataHandler, Panel, DataAccessor
from command.response import Response
from model.bigbed import get_bigwig_stats, get_bigwig
from model.chromosome import Chromosome
from model.transcriptfile import TranscriptFileLine
from .numbers import delta, zigzag, lesqlite2, compress, classify
from data.util import domino_series_expand

"""
Attributes:
    SCALE (int)
"""

SCALE = 1000



def get_variant_stats(data_accessor: DataAccessor, chrom: Chromosome, panel: Panel) -> Response:
    """

    Args:
        data_accessor (object):
        chrom (object):
        panel (object):

    Returns:

    """
    item = chrom.item_path("variant-summary")
    (data, start, end) = get_bigwig_stats(data_accessor, item, panel.start, panel.end, "max", nBins=500)
    data = [0.0 if x is None else x for x in data]
    length = len(data)
    if length == 0:
        length = 1
    step = int((end - start) * SCALE / length)
    if step == 0:
        step = SCALE
    data = bytearray([round(x) for x in data])
    out = {
        "values": compress(lesqlite2(zigzag(delta(data)))),
        "range": compress(lesqlite2([start, end, step]))
    }
    return Response(5, {'data': out})


def get_variant_exact(data_accessor: DataAccessor, chrom: Chromosome, panel: Panel) -> Response:
    """

    Args:
        data_accessor (object):
        chrom (object):
        panel (object):

    Returns:
        Response object

    """
    item = chrom.item_path("variant-summary")
    (data, start, end) = get_bigwig(data_accessor, item, panel.start, panel.end)
    data = [0.0 if x is None else x for x in data]
    length = len(data)
    if length == 0:
        length = 1
    step = int((end - start) * SCALE / length)
    if step == 0:
        step = SCALE
    data = bytearray([round(x) for x in data])
    out = {
        "values": compress(lesqlite2(zigzag(delta(data)))),
        "range": compress(lesqlite2([start, end, step]))
    }
    return Response(5, {'data': out})


def get_variant(data_accessor: DataAccessor, chrom: Chromosome, panel: Panel) -> Response:
    """

    Args:
        data_accessor (object):
        chrom (object):
        panel (object):

    Returns:
        Response object
    """
    if panel.end - panel.start > 1000:
        return get_variant_stats(data_accessor, chrom, panel)
    else:
        return get_variant_exact(data_accessor, chrom, panel)


class VariantDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel) -> Response:
        """

        Args:
            data_accessor (object):
            panel (object):

        Returns:
            Response object
        """
        chrom = data_accessor.data_model.stick(data_accessor, panel.stick)
        if chrom is None:
            return Response(1, "Unknown chromosome {0}".format(panel.stick))
        return get_variant(data_accessor, chrom, panel)
