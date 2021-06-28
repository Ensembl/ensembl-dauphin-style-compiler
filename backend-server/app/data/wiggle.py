import collections
import logging
from command.coremodel import DataHandler, Panel, DataAccessor, Response
from model.bigbed import get_bigbed_data, get_bigwig_data
from model.chromosome import Chromosome
from model.transcriptfile import TranscriptFileLine
from .numbers import delta, zigzag, lesqlite2, compress, classify

SCALE = 4

def get_wiggle(chrom: Chromosome, panel: Panel) -> Response:
    path = chrom.file_path("gc","gc.bw")
    data = get_bigwig_data(path,chrom,panel.start,panel.end)
    data = bytearray([round(x/SCALE) for x in data])
    out = {
        "values": compress(lesqlite2(zigzag(delta(data)))),
        "range": compress(lesqlite2([panel.start,panel.end]))
    }
    return Response(5,{ 'data': out })

class WiggleDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel) -> Response:
        chrom = data_accessor.data_model.sticks.get(panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        return get_wiggle(chrom,panel)
