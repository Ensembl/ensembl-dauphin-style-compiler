from command.coremodel import DataHandler, Panel, DataAccessor
from command.response import Response
from command.exceptionres import DataException
from model.bigbed import get_bigwig_stats, get_bigwig
from model.chromosome import Chromosome
from data.v14.dataalgorithm import data_algorithm

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


class VariantDataHandler2(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope, accept) -> Response:
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if chrom == None:
            raise DataException("Unknown chromosome {0}".format(panel.stick))
        return get_variant(data_accessor,chrom,panel)
