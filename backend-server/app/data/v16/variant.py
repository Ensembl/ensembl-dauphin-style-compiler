import logging
from command.coremodel import DataHandler, Panel, DataAccessor
from command.response import Response
from command.exceptionres import DataException
from model.bigbed import get_bigwig_stats, get_bigwig, get_bigbed
from model.chromosome import Chromosome
from data.v16.dataalgorithm import data_algorithm

SCALE = 4000

def get_variant_stats(data_accessor: DataAccessor, chrom: Chromosome, panel: Panel) -> Response:
    item = chrom.item_path("variant-summary")
    (data, start, end) = get_bigwig_stats(data_accessor, item, panel.start, panel.end, "max", nBins=500)
    data = [0.0 if x is None else x for x in data]
    length = len(data)
    if length == 0:
        length = 1
    step = int((end - start) * SCALE / length)
    if step == 0:
        step = SCALE
    data = bytearray([round(x) for x in data])
    return {
        "values": data_algorithm("NDZRL",data),
        "range": data_algorithm("NRL",[start,end,step])
    }

def get_variant_exact(data_accessor: DataAccessor, chrom: Chromosome, panel: Panel) -> Response:
    item = chrom.item_path("variant-summary")
    (data, start, end) = get_bigwig(data_accessor, item, panel.start, panel.end)
    data = [0.0 if x is None else x for x in data]
    length = len(data)
    if length == 0:
        length = 1
    step = int((end - start) * SCALE / length)
    if step == 0:
        step = SCALE
    data = bytearray([round(x) for x in data])
    return {
        "values": data_algorithm("NDZRL",data),
        "range": data_algorithm("NRL",[start,end,step])
    }


def get_variant(data_accessor: DataAccessor, chrom: Chromosome, panel: Panel) -> Response:
    if panel.end - panel.start > 1000:
        return get_variant_stats(data_accessor, chrom, panel)
    else:
        return get_variant_exact(data_accessor, chrom, panel)


class VariantSummaryDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope, accept) -> Response:
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if chrom == None:
            raise DataException("Unknown chromosome {0}".format(panel.stick))
        return get_variant(data_accessor,chrom,panel)

def get_variant_labels(data_accessor: DataAccessor, chrom: Chromosome, panel: Panel) -> Response:
    item = chrom.item_path("variant-labels")
    try:
        data = get_bigbed(data_accessor,item,panel.start,panel.end)
        starts = []
        lengths = []
        ids = []
        varieties = []
        refs = []
        alts = []
        severities = []
        for (start,end,rest) in data:
            rest = rest.split()
            starts.append(start)
            lengths.append(end-start)
            ids.append(rest[0])
            varieties.append(rest[1])
            refs.append(rest[2])
            alts.append(rest[3])
            severities.append(int(rest[4]))
    except Exception as e:
        logging.error(e)
    return {
        "start": data_algorithm("NDZRL",starts),
        "length": data_algorithm("NDZRL",lengths),
        "id": data_algorithm("SZ",ids),
        "variety": data_algorithm("SYRLZ",varieties),
        "ref": data_algorithm("SYRLZ",refs),
        "alt": data_algorithm("SYRLZ",alts),
        "severity": data_algorithm("NRL",severities)
    }

class VariantLabelsDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope, accept) -> Response:
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if chrom == None:
            raise DataException("Unknown chromosome {0}".format(panel.stick))
        return get_variant_labels(data_accessor,chrom,panel)
