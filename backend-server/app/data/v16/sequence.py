from typing import Dict
from command.coremodel import DataHandler, Panel, DataAccessor
from command.response import Response
from model.chromosome import Chromosome
from command.exceptionres import DataException
from data.v16.dataalgorithm import data_algorithm


def retrieve_range(data_accessor: DataAccessor, chrom: Chromosome, panel: Panel):
    item = chrom.item_seq_path("")
    resolver = data_accessor.resolver.get(item)
    return resolver.get(panel.start, panel.end - panel.start).decode("utf-8")


def sequence_blocks8(out: Dict[str, bytes], data_accessor: DataAccessor, chrom: Chromosome, panel: Panel, dummy: bool):
    line = ""
    if not dummy:
        line = list(" " * (panel.end - panel.start + 2))
        sequence = retrieve_range(data_accessor, chrom, panel)
        for (offset, letter) in enumerate(sequence):
            line[offset] = letter if letter in "CGAT" else " "
    out['sequence'] = data_algorithm("SC", "".join(line))
    out['sequence_start'] = data_algorithm("NRL", [panel.start])
    return out


class ZoomedSeqDataHandler16(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope) -> Response:
        chrom = data_accessor.data_model.stick(panel.stick)
        if chrom == None:
            raise DataException(1, "Unknown chromosome {0}".format(panel.stick))
        out = {}
        return sequence_blocks8(out, data_accessor, chrom, panel, False)
