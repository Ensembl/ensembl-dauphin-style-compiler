from command.coremodel import DataHandler, Panel, DataAccessor
from command.response import Response
from command.exceptionres import DataException
from model.bigbed import get_bigbed
from model.chromosome import Chromosome
from data.v16.dataalgorithm import data_algorithm
import logging

feature_type_index_map = {
    "Promoter": 0,
    "Enhancer": 1,
    "Open chromatin": 2,
    "CTCF Binding Site": 3,
    "TF binding": 4
}

def get_regulation_data(
    data_accessor: DataAccessor,
    chrom: Chromosome,
    panel: Panel,
) -> Response:
    try:
        item = chrom.item_path("regulation")
        data = get_bigbed(data_accessor, item, panel.start, panel.end)
        starts = []
        lengths = []
        ids = []
        sticks = []
        thick_starts = []
        thick_ends = []
        feature_types = []
        feature_type_indices = []
        for (start, end, rest) in data:
            rest = rest.split("\t") # Regulation team uses tabs as separators in their bigbeds
            id = rest[0]
            thick_start = int(rest[3])
            thick_end = int(rest[4])
            feature_type = rest[9]
            feature_type_index = feature_type_index_map[feature_type]

            sticks.append(chrom.name)
            starts.append(start)
            lengths.append(end - start)
            ids.append(id)
            thick_starts.append(thick_start)
            thick_ends.append(thick_end)
            feature_types.append(feature_type)
            feature_type_indices.append(feature_type_index)

    except Exception as e:
        logging.error(e)
    return {
        "stick": data_algorithm("SZ", sticks),
        "start": data_algorithm("NDZRL", starts),
        "length": data_algorithm("NDZRL", lengths),
        "id": data_algorithm("SZ", ids),
        "thick_start": data_algorithm("NDZRL", thick_starts),
        "thick_end": data_algorithm("NDZRL", thick_ends),
        "feature_type": data_algorithm("SZ", feature_types),
        "feature_type_index": data_algorithm("NDZRL", feature_type_indices),
    }



class RegulationDataHandler(DataHandler):
    def process_data(
        self, data_accessor: DataAccessor, panel: Panel, scope, accept
    ) -> Response:
        chrom = data_accessor.data_model.stick(data_accessor, panel.stick)
        if chrom == None:
            raise DataException("Unknown chromosome {0}".format(panel.stick))
        return get_regulation_data(data_accessor, chrom, panel)
