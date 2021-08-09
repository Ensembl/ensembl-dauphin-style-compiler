import collections
import logging
from command.coremodel import DataHandler, Panel, DataAccessor, Response
from model.bigbed import get_bigwig_stats, get_bigwig
from model.chromosome import Chromosome
from model.transcriptfile import TranscriptFileLine
from .numbers import delta, zigzag, lesqlite2, compress, classify

SCALE=1000

def get_variant_stats(data_accessor : DataAccessor, chrom: Chromosome, panel: Panel) -> Response:
    item = chrom.item_path("variant-summary")
    data = get_bigwig_stats(data_accessor,item,panel.start,panel.end,"max",nBins=250)
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

def get_variant_exact(data_accessor : DataAccessor, chrom: Chromosome, panel: Panel) -> Response:
    item = chrom.item_path("variant-summary")
    data = get_bigwig(data_accessor,item,panel.start,panel.end)
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

def get_variant(data_accessor : DataAccessor, chrom: Chromosome, panel: Panel) -> Response:
    if panel.end-panel.start > 5000:
        return get_variant_stats(data_accessor,chrom,panel)
    else:
        return get_variant_exact(data_accessor,chrom,panel)

class VariantDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel) -> Response:
        chrom = data_accessor.data_model.sticks.get(panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        return get_variant(data_accessor,chrom,panel)
