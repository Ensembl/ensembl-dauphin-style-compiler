import logging
from command.coremodel import DataHandler, Panel, DataAccessor
from command.response import Response
from model.bigbed import get_bigwig_stats, get_bigwig, get_bigbed
from model.datalocator import AccessItem
from data.v16.dataalgorithm import data_algorithm

SCALE = 4000


def get_variant_stats(
    data_accessor: DataAccessor, access_item: AccessItem, panel: Panel
) -> dict[str, bytearray]:
    (data, start, end) = get_bigwig_stats(
        data_accessor, access_item, panel.start, panel.end, "max", nBins=500
    )
    data = [0.0 if x is None else x for x in data]
    length = len(data)
    if length == 0:
        length = 1
    step = int((end - start) * SCALE / length)
    if step == 0:
        step = SCALE
    data = bytearray([round(x) for x in data])
    return {
        "values": data_algorithm("NDZRL", data),
        "range": data_algorithm("NRL", [start, end, step]),
    }


def get_variant_exact(
    data_accessor: DataAccessor, access_item: AccessItem, panel: Panel
) -> dict[str, bytearray]:
    (data, start, end) = get_bigwig(data_accessor, access_item, panel.start, panel.end)
    data = [0.0 if x is None else x for x in data]
    length = len(data)
    if length == 0:
        length = 1
    step = int((end - start) * SCALE / length)
    if step == 0:
        step = SCALE
    try:
        data = bytearray([round(x) for x in data])
    except ValueError as e:
        logging.error(f"Unexpected data in {access_item.item_suffix()}: {e}")
        data = bytearray([0]*length)
    return {
        "values": data_algorithm("NDZRL", data),
        "range": data_algorithm("NRL", [start, end, step]),
    }


def get_variant(
    data_accessor: DataAccessor, panel: Panel, filename: str
) -> dict:
    item = panel.get_chrom(data_accessor).item_path(filename)
    if panel.end - panel.start > 1000:
        return get_variant_stats(data_accessor, item, panel)
    else:
        return get_variant_exact(data_accessor, item, panel)


class VariantSummaryDataHandler(DataHandler):
    def process_data(
        self, data_accessor: DataAccessor, panel: Panel, scope: dict, accept: str
    ) -> dict:
        return get_variant(data_accessor, panel, self.get_datafile(scope))


def get_variant_labels(
    data_accessor: DataAccessor, panel: Panel, filename: str, start: str | None=None, id: str | None=None
) -> dict[str, bytearray]:
    chrom = panel.get_chrom(data_accessor)
    access_item = chrom.item_path(filename)
    try:
        if start: # only start is needed to fetch the variant
            panel.start = int(start)-1
            panel.end = panel.start+2
        data = get_bigbed(data_accessor, access_item, panel.start, panel.end)
        starts = []
        lengths = []
        ids = []
        varieties = []
        groups = []
        consequence = []
        chromosomes = []
        alleles = []
        for start, end, rest in data:
            rest = rest.split()
            chromosomes.append(chrom.name)
            starts.append(start)
            lengths.append(end - start)
            ids.append(rest[0])
            varieties.append(rest[1])
            alleles.append(allele_sequence(rest[2], rest[3]))
            groups.append(int(rest[4]))
            consequence.append(rest[5])
    except Exception as e:
        logging.error(e)
    return {
        "chromosome": data_algorithm("SZ", chromosomes),
        "start": data_algorithm("NDZRL", starts),
        "length": data_algorithm("NDZRL", lengths),
        "id": data_algorithm("SZ", ids),
        "variety": data_algorithm("SYRLZ", varieties),
        "alleles": data_algorithm("SYRLZ", alleles),
        "group": data_algorithm("NRL", groups),
        "consequence": data_algorithm("SYRLZ", consequence),
    }


def allele_sequence(ref: str, alts: str) -> str:
    combined_sequence = ref + ' ' + alts
    if len(combined_sequence) > 18:
        truncated_sequence = combined_sequence[0:18] + '…'
        return truncated_sequence
    return combined_sequence

class VariantLabelsDataHandler(DataHandler):
    def process_data(
        self, data_accessor: DataAccessor, panel: Panel, scope: dict, accept: str
    ) -> dict:
        return get_variant_labels(
            data_accessor, panel, self.get_datafile(scope), self.get_scope(scope,"start"), self.get_scope(scope,"id")
        )
