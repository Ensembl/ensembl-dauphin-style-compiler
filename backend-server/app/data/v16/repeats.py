from command.coremodel import DataHandler, Panel, DataAccessor
from data.v16.dataalgorithm import data_algorithm
from model.bigbed import get_bigbed

def get_repeat_details(
        data_accessor: DataAccessor, panel: Panel, filename: str
    ) -> dict[str,bytearray]:
    item = panel.get_chrom(data_accessor).item_path(filename)
    data = get_bigbed(data_accessor, item, panel.start, panel.end)
    starts = []
    ends = []
    strands = []
    analyses = []
    repeatNames = []
    repeatClasses = []
    for (start, end, rest) in data:
        (strand, analysis, repeatName, repeatClass) = rest.split("\t")
        starts.append(start)
        ends.append(end)
        strands.append(strand)
        analyses.append(analysis)
        repeatNames.append(repeatName)
        repeatClasses.append(repeatClass)

    return {
        "start": data_algorithm("NDZRL", starts),
        "end": data_algorithm("NDZRL", ends),
        "strand": data_algorithm("SZ", strands),
        "analyses": data_algorithm("SZ", analyses),
        "repeatNames": data_algorithm("SZ", repeatNames),
        "repeatClasses": data_algorithm("SZ", repeatClasses),
    }

class RepeatsDataHandler(DataHandler):
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
    ) -> dict[str,bytearray]:
        return get_repeat_details(data_accessor, panel, self.get_datafile(scope))
