from command.coremodel import DataHandler, Panel, DataAccessor
from data.v16.dataalgorithm import data_algorithm
from model.bigbed import get_bigbed


def get_features(
    data_accessor: DataAccessor, panel: Panel, filename: str
) -> dict[str, bytearray]:
    chrom = panel.get_chrom(data_accessor)
    data = get_bigbed(data_accessor, chrom.item_path(filename), panel.start, panel.end)
    chrs = []
    starts = []
    ends = []
    strands = []
    analyses = []
    scores = []

    for (start, end, rest) in data:
        (strand, analysis, score) = rest.split("\t")
        chrs.append(chrom.name)
        starts.append(start)
        ends.append(end)
        strands.append(strand)
        analyses.append(analysis)
        scores.append(int(float(score)))

    return {
        "chr": data_algorithm("SZ", chrs),
        "start": data_algorithm("NDZRL", starts),
        "end": data_algorithm("NDZRL", ends),
        "strand": data_algorithm("SZ", strands),
        "analysis": data_algorithm("SZ", analyses),
        "score": data_algorithm("NZRL", scores),
    }


class FeaturesDataHandler(DataHandler):
    """
    Handle a request for compara bigbed data (conserved elements).

    Args:
        data_accessor (DataAccessor): The means of accessing data
        panel (Panel): The panel (ie genomic location, scale) we want
        scope: extra scope args (here used for datafile name)

    Returns: A data dict (payload for Response object)
    """

    def process_data(
        self, data_accessor: DataAccessor, panel: Panel, scope: dict, accept: str
    ) -> dict[str, bytearray]:
        return get_features(data_accessor, panel, self.get_datafile(scope))
