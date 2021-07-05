import collections
import logging
from command.coremodel import DataHandler, Panel, DataAccessor, Response
from model.bigbed import get_bigbed_data, get_bigwig_data
from model.chromosome import Chromosome
from model.transcriptfile import TranscriptFileLine
from .numbers import delta, zigzag, lesqlite2, compress, classify
from .util import starts_and_ends

SCALE = 4

def get_contig(chrom: Chromosome, panel: Panel) -> Response:
    path = chrom.file_path("contigs","contigs.bb")
    data = get_bigbed_data(path,chrom,panel.start,panel.end)
    positions = []
    senses = []
    for line in data:
        (contig_start, contig_end, rest) = line
        (name, value, sense) = rest.split("\t")
        positions.append((contig_start,contig_end))
        senses.append(sense=='+')
    out = {
        "sense": compress(lesqlite2(senses))
    }
    starts_and_ends(out,positions,"contig")
    return Response(5,{ 'data': out })

class ContigDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel) -> Response:
        chrom = data_accessor.data_model.sticks.get(panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        return get_contig(chrom,panel)
