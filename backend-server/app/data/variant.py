import collections
import logging
from command.coremodel import DataHandler, Panel, DataAccessor, Response
from model.bigbed import get_bigwig_stats, get_bigwig_data
from model.chromosome import Chromosome
from model.transcriptfile import TranscriptFileLine
from .numbers import delta, zigzag, lesqlite2, compress, classify

SCALE=1000

def get_variant_stats(chrom: Chromosome, panel: Panel) -> Response:
    path = chrom.file_path("variants","summary.{}.bw".format(chrom.name))
    data = get_bigwig_stats(path,chrom,panel.start,panel.end,"max")
    data = [ 0.0 if x is None else x for x in data ]
    length = len(data)
    if length == 0:
        length = 1
    step = int((panel.end-panel.start)*SCALE/length)
    if step == 0:
        step = SCALE
    data = bytearray([round(x) for x in data])
    out = {
        "values": compress(lesqlite2(zigzag(delta(data)))),
        "range": compress(lesqlite2([panel.start,panel.end,step]))
    }
    return Response(5,{ 'data': out })

def get_variant_exact(chrom: Chromosome, panel: Panel) -> Response:
    path = chrom.file_path("variants","summary.{}.bw".format(chrom.name))
    data = get_bigwig_data(path,chrom,panel.start,panel.end)
    data = [ 0.0 if x is None else x for x in data ]
    length = len(data)
    if length == 0:
        length = 1
    step = int((panel.end-panel.start)*SCALE/length)
    if step == 0:
        step = SCALE
    data = bytearray([round(x) for x in data])
    out = {
        "values": compress(lesqlite2(zigzag(delta(data)))),
        "range": compress(lesqlite2([panel.start,panel.end,step]))
    }
    return Response(5,{ 'data': out })

def get_variant(chrom: Chromosome, panel: Panel) -> Response:
    if panel.end-panel.start > 5000:
        return get_variant_stats(chrom,panel)
    else:
        return get_variant_exact(chrom,panel)

class VariantDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel) -> Response:
        chrom = data_accessor.data_model.sticks.get(panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        return get_variant(chrom,panel)
