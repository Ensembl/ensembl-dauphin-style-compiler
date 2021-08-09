import logging
from typing import Dict
from command.coremodel import DataHandler, Panel, DataAccessor
from command.response import Response
from model.chromosome import Chromosome
from .numbers import delta, zigzag, lesqlite2, compress, classify
from .util import classified_numbers


def sequence_blocks(out: Dict[str,bytes], chrom: Chromosome, panel: Panel, dummy: bool):
    file_path = chrom.file_path("seqs",chrom.seq_hash)
    starts = []
    letters = []
    if not dummy:
        with open(file_path) as f:
            f.seek(panel.start)
            sequence = f.read(panel.end-panel.start)
            for (offset,letter) in enumerate(sequence):
                starts.append(panel.start+offset)
                letters.append(letter if letter in "CGAT" else "")
    out['seq_starts'] = compress(lesqlite2(zigzag(delta(starts))))
    classified_numbers(out,letters,"seq")

class ZoomedSeqDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel) -> Response:
        chrom = data_accessor.data_model.sticks.get(panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        out = {}
        sequence_blocks(out,chrom,panel,False)
        return Response(5,{ 'data': out })
