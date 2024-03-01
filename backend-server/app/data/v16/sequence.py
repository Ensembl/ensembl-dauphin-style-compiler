from command.coremodel import DataHandler, Panel, DataAccessor
from data.v16.dataalgorithm import data_algorithm


def retrieve_range(data_accessor: DataAccessor, panel: Panel) -> str:
    item = panel.get_chrom(data_accessor).item_seq_path("")
    resolver = data_accessor.resolver.get(item)
    return resolver.get(panel.start, panel.end - panel.start).decode("utf-8")


def sequence_blocks8(
        data_accessor: DataAccessor, panel: Panel, dummy: bool
    ) -> dict[str, bytes]:
    line = [""]
    out = {}
    if not dummy:
        line = list(" " * (panel.end - panel.start + 2))
        sequence = retrieve_range(data_accessor, panel)
        for (offset, letter) in enumerate(sequence):
            line[offset] = letter if letter in "CGAT" else " "
    out['sequence'] = data_algorithm("SC", "".join(line))
    out['sequence_start'] = data_algorithm("NRL", [panel.start])
    return out


class ZoomedSeqDataHandler16(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope: dict, accept: str) -> dict:
        return sequence_blocks8(data_accessor, panel, False)
