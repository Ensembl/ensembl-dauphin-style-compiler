import logging
from typing import Dict
from command.coremodel import DataHandler, Panel, DataAccessor
from command.response import Response
from model.chromosome import Chromosome
from command.exceptionres import DataException
from .numbers import lesqlite2, compress

def retrieve_range(data_accessor: DataAccessor,chrom: Chromosome, panel: Panel):
    item = chrom.item_seq_path("seqs")
    resolver = data_accessor.resolver.get(item)
    return resolver.get(panel.start,panel.end-panel.start).decode("utf-8")

def sequence_blocks8(out: Dict[str,bytes], data_accessor: DataAccessor, chrom: Chromosome, panel: Panel, dummy: bool):
    line = ""
    if not dummy:
        line = list(" " * (panel.end-panel.start+2))
        sequence = retrieve_range(data_accessor,chrom,panel)
        logging.error("line len = {0} seq len {1}".format(len(line),len(sequence)))
        for (offset,letter) in enumerate(sequence):
            line[offset] = letter if letter in "CGAT" else " "
    out['sequence'] = compress("".join(line))
    out['sequence_start'] = compress(lesqlite2([panel.start]))

class ZoomedSeqDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope, accept) -> Response:
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if chrom == None:
            raise DataException(1,"Unknown chromosome {0}".format(panel.stick))
        out = {}
        sequence_blocks8(out,data_accessor,chrom,panel,False)
        return out
