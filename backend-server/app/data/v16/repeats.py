from command.coremodel import DataHandler, Panel, DataAccessor
from data.v16.dataalgorithm import data_algorithm
from model.bigbed import get_bigwig_stats, get_bigbed_fields

SCALE = 1000

def get_repeat_details(
        data_accessor: DataAccessor, panel: Panel, filename: str
    ) -> dict[str,bytearray]:
    fields = get_bigbed_fields(
        data_accessor, panel, filename,
        ["strand", "analysis", "name", "class", "type"],
    )

    return {
        "chr": data_algorithm("SZ", fields["chr"]),
        "start": data_algorithm("NDZRL", fields["start"]),
        "end": data_algorithm("NDZRL", fields["end"]),
        "strand": data_algorithm("SZ", fields["strand"]),
        "analysis": data_algorithm("SZ", fields["analysis"]),
        "name": data_algorithm("SZ", fields["name"]),
        "class": data_algorithm("SZ", fields["class"]),
        "type": data_algorithm("SZ", fields["type"]),
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
