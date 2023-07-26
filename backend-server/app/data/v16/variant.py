import logging
from typing import Optional, Tuple
from model.datalocator import AccessItem
from command.coremodel import DataHandler, Panel, DataAccessor
from command.response import Response
from command.exceptionres import DataException
from model.bigbed import get_bigwig_stats, get_bigwig, get_bigbed
from model.chromosome import Chromosome
from data.v16.dataalgorithm import data_algorithm
from ncd import NCDRead

SCALE = 4000


def get_variant_stats(
    data_accessor: DataAccessor, chrom: Chromosome, panel: Panel, track_id: str
) -> Response:
    item = chrom.item_path(track_id)
    (data, start, end) = get_bigwig_stats(
        data_accessor, item, panel.start, panel.end, "max", nBins=500
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
    data_accessor: DataAccessor, chrom: Chromosome, panel: Panel, track_id: str
) -> Response:
    item = chrom.item_path(track_id)
    (data, start, end) = get_bigwig(data_accessor, item, panel.start, panel.end)
    data = [0.0 if x is None else x for x in data]
    length = len(data)
    if length == 0:
        length = 1
    step = int((end - start) * SCALE / length)
    if step == 0:
        step = SCALE

    logging.info('HERE!!!!!')
    logging.info(['PANEL START AND END', panel.start, panel.end])
    logging.info(['START AND END', start, end])
    logging.info(data)

    data = bytearray([round(x) for x in data])
    return {
        "values": data_algorithm("NDZRL", data),
        "range": data_algorithm("NRL", [start, end, step]),
    }


def get_variant(
    data_accessor: DataAccessor, chrom: Chromosome, panel: Panel, track_id: str
) -> Response:
    if panel.end - panel.start > 1000:
        return get_variant_stats(data_accessor, chrom, panel, track_id)
    else:
        return get_variant_exact(data_accessor, chrom, panel, track_id)


class VariantSummaryDataHandler(DataHandler):
    def __init__(self, track_id):
        self.track_id = track_id + "-summary"

    def process_data(
        self, data_accessor: DataAccessor, panel: Panel, scope, accept
    ) -> Response:
        chrom = data_accessor.data_model.stick(data_accessor, panel.stick)
        if chrom == None:
            raise DataException("Unknown chromosome {0}".format(panel.stick))
        return get_variant(data_accessor, chrom, panel, self.track_id)


def get_approx_location(data_accessor: DataAccessor, genome: str, id):
    # TODO: Add update files and :variant: back in; add limited size for none
    # replace with canonical form for focus lookup
    genome = data_accessor.data_model.canonical_genome_id(genome)
    if genome != None:
        species = data_accessor.data_model.species(genome)
        key = "focus:variant:{}:{}".format(species.wire_id, id)
        accessor = data_accessor.resolver.get(AccessItem("jump", species.genome_id))
        jump_ncd = NCDRead(accessor.ncd())
        value = jump_ncd.get(key.encode("utf-8"))
        if value != None:
            parts = value.decode("utf-8").split("\t")
            if len(parts) >= 3:
                on_stick = "{}:{}".format(genome, parts[0])
                return (on_stick, int(parts[1]), int(parts[2]))
    return (None, None, None)


def update_panel_from_id(
    data_accessor: DataAccessor, panel: Panel, for_id: Tuple[str, str]
):
    (stick, start, end) = get_approx_location(data_accessor, for_id[0], for_id[1])
    if stick is not None:
        panel.stick = stick
        panel.start = start
        panel.end = end
    else:
        # will be rejexted by FE anyway, so keep it short
        logging.warn("HELP!")
        panel.end = panel.start + 1


def get_variant_labels(
    data_accessor: DataAccessor,
    chrom: Chromosome,
    panel: Panel,
    filename: str,
    for_id: Optional[Tuple[str, str]],
) -> Response:
    if for_id is not None:
        update_panel_from_id(data_accessor, panel, for_id)
    item = chrom.item_path(filename)
    try:
        data = get_bigbed(data_accessor, item, panel.start, panel.end)
        starts = []
        lengths = []
        ids = []
        varieties = []
        severities = []
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
            severities.append(int(rest[4]))
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
        "severity": data_algorithm("NRL", severities),
        "consequence": data_algorithm("SYRLZ", consequence),
    }

def allele_sequence(ref: str, alts: str) -> str:
    combined_sequence = ref + '/' + alts
    if len(combined_sequence) > 18:
        truncated_sequence = combined_sequence[0:18] + '…'
        return truncated_sequence
    return combined_sequence


def for_id(scope):
    genome_id = scope.get("genome")
    if genome_id is not None and len(genome_id) == 0:
        genome_id = None
    obj_id = scope.get("id")
    if obj_id is not None and len(obj_id) == 0:
        obj_id = None
    if genome_id is not None and obj_id is not None:
        return (genome_id[0], obj_id[0])
    else:
        return None


class VariantLabelsDataHandler(DataHandler):
    def __init__(self, track_id: Optional[str] = None):
        self.filename = "variant-labels" + ("-" + track_id if track_id else "")

    def process_data(
        self, data_accessor: DataAccessor, panel: Panel, scope, accept
    ) -> Response:
        chrom = data_accessor.data_model.stick(data_accessor, panel.stick)
        if chrom == None:
            raise DataException("Unknown chromosome {0}".format(panel.stick))
        return get_variant_labels(
            data_accessor, chrom, panel, self.filename, for_id(scope)
        )
