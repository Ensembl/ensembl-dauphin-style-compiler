from command.coremodel import DataHandler, Panel, DataAccessor
from data.v16.dataalgorithm import data_algorithm
from model.bigbed import get_bigbed, get_bigwig_stats

SCALE = 1000

def get_repeat_details(
        data_accessor: DataAccessor, panel: Panel, filename: str
    ) -> dict[str,bytearray]:
    chrom = panel.get_chrom(data_accessor)
    data = get_bigbed(data_accessor, chrom.item_path(filename), panel.start, panel.end)
    chrs = [chrom.name] * len(data)
    starts = []
    ends = []
    strands = []
    analyses = []
    names = []
    classes = []
    types = []

    for (start, end, rest) in data:
        (strand, analysis, name, repclass, type) = rest.split("\t")
        starts.append(start)
        ends.append(end)
        strands.append(strand)
        analyses.append(analysis)
        classes.append(repclass)
        names.append(name)
        types.append(type)

    return {
        "chr": data_algorithm("SZ", chrs),
        "start": data_algorithm("NDZRL", starts),
        "end": data_algorithm("NDZRL", ends),
        "strand": data_algorithm("SZ", strands),
        "analysis": data_algorithm("SZ", analyses),
        "name": data_algorithm("SZ", names),
        "class": data_algorithm("SZ", classes),
        "type": data_algorithm("SZ", types),
    }

def get_repeat_density(
        data_accessor: DataAccessor, panel: Panel, filename: str
    ) -> dict[str, bytearray]:
    item = panel.get_chrom(data_accessor).item_path(filename)
    (data, start, end) = get_bigwig_stats(
        data_accessor, item, panel.start, panel.end, consolidation="mean", nBins=500
    )
    data = [0.0 if x is None else x for x in data]
    length = len(data)
    if length == 0:
        length = 1
    step = int((end - start) * SCALE / length)
    if step == 0:
        step = SCALE
    scaled = bytearray([min(255, max(0, round(x * 255))) for x in data])
    return {
        "values": data_algorithm("NDZRL", scaled),
        "range": data_algorithm("NRL", [start, end, step]),
    }

class RepeatsDataHandler(DataHandler):
    """
    Handle a request for repeat elements data.

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

class RepeatSummaryDataHandler(DataHandler):
    """
    Handle a request for repeats summary data.

    Args:
        data_accessor (DataAccessor): The means of accessing data
        panel (Panel): Requested genomic location and scale
        scope: extra scope args (here used for datafile name)
    
    Returns: A data dict (values, range) for Response object
    """
    def process_data(
        self, data_accessor: DataAccessor, panel: Panel, scope: dict, accept: str
    ) -> dict[str,bytearray]:
        filename = self.get_scope(scope, "datafile")
        # placeholder empty data for tracks without bigwig datafile
        if not filename:
            return {
                "values": data_algorithm("NDZRL", []),
                "range": data_algorithm("NRL", [panel.start, panel.end, SCALE]),
            }
        return get_repeat_density(data_accessor, panel, filename)
