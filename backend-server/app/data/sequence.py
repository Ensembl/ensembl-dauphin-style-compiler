import logging
from typing import Dict
from command.coremodel import DataHandler, Panel, DataAccessor
from command.response import Response
from model.chromosome import Chromosome
from .numbers import delta, zigzag, lesqlite2, compress
from .util import classified_numbers

def retrieve_range(data_accessor: DataAccessor,chrom: Chromosome, panel: Panel):
    item = chrom.item_seq_path("seqs")
    resolver = data_accessor.resolver.get(item)
    return resolver.get(panel.start,panel.end-panel.start).decode("utf-8")

def sequence_blocks(out: Dict[str,bytes], data_accessor: DataAccessor, chrom: Chromosome, panel: Panel, dummy: bool):
    starts = []
    letters = []
    if not dummy:
        sequence = retrieve_range(data_accessor,chrom,panel)
        for (offset,letter) in enumerate(sequence):
            starts.append(panel.start+offset)
            letters.append(letter if letter in "CGAT" else "")
    out['seq_starts'] = compress(lesqlite2(zigzag(delta(starts))))
    classified_numbers(out,letters,"seq")

class ZoomedSeqDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope) -> Response:
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        out = {}
        sequence_blocks(out,data_accessor,chrom,panel,False)
        return Response(5,{ 'data': out })
