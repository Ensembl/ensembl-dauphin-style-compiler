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
    # Awkwardly, issing data seems tobe treated as 0.0 by the reader.
    # Need also to mask adjacentvalues as averages are affected
    data = [ 0.0 if x is None else x for x in data ]
    present = []
    for i in range(0,len(data)):
        missing = data[i]<= 0.0 or (i>0 and data[i-1]<=0.0) or (i<len(data)-1 and data[i+1]<=0.0)
        present.append(not missing)    
    present = bytearray(present)
    data = bytearray([round(x/SCALE) for x in data])
    out = {
        "values": compress(lesqlite2(zigzag(delta(data)))),
        "present": compress(present),
        "range": compress(lesqlite2([panel.start,panel.end]))
    }
    return Response(5,{ 'data': out })

class WiggleDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel) -> Response:
        chrom = data_accessor.data_model.sticks.get(panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        return get_wiggle(chrom,panel)
