import logging
from command.coremodel import DataHandler, Panel, DataAccessor
from command.response import Response
from model.bigbed import get_bigwig_stats, get_bigwig, get_bigbed_fields
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
    data_accessor: DataAccessor, panel: Panel, filename: str, start: str | None=None, sv: bool=False
) -> dict[str, bytearray]:
    try:
        if start: # only start is needed to fetch the variant
            panel.start = int(start)-1
            panel.end = panel.start+2
        meta_fields = ["chr", "start", "end", "id", "variety", "ref", "alt", "group", "consequence"]
        if sv:
            meta_fields += ["extent"]
        fields = get_bigbed_fields(
            data_accessor, panel, filename,
            meta_fields
        )
        lengths = [end - start for start, end in zip(fields["start"], fields["end"])]
        alleles = [allele_sequence(ref, alt) for ref, alt in zip(fields["ref"], fields["alt"])]
        groups = [int(group) for group in fields["group"]]
        if sv:
            groups = [max(0, group-10) for group in groups] # shift down to 0-5 for track coloring
    except Exception as e:
        logging.error(e)
    payload = {
        "chromosome": data_algorithm("SZ", fields["chr"]),
        "start": data_algorithm("NDZRL", fields["start"]),
        "length": data_algorithm("NDZRL", lengths),
        "id": data_algorithm("SZ", fields["id"]),
        "variety": data_algorithm("SYRLZ", fields["variety"]),
        "alleles": data_algorithm("SYRLZ", alleles),
        "group": data_algorithm("NRL", groups),
        "consequence": data_algorithm("SYRLZ", fields["consequence"]),
    }
    if sv:
        extent = [int(extent) for extent in fields["extent"]]
        payload["extent"] = data_algorithm("NDZRL", extent)
    return payload


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
            data_accessor, panel, self.get_datafile(scope), self.get_scope(scope,"start")
        )

class StructuralVariantLabelsDataHandler(DataHandler):
    def process_data(
        self, data_accessor: DataAccessor, panel: Panel, scope: dict, accept: str
    ) -> dict:
        return get_variant_labels(
            data_accessor, panel, self.get_datafile(scope), self.get_scope(scope,"start"), True
        )
