import math

from command.coremodel import DataHandler, Panel, DataAccessor
from data.v16.dataalgorithm import data_algorithm
from model.bigbed import get_bigwig_stats, get_bigwig

def get_wiggle_data( data_accessor: DataAccessor, panel: Panel, data_file: str, data_range: tuple[int,int]=(0,100)) -> dict:
    """
    Use the DataAccessor provided to access the wiggle data for a given location.

    Args:
        data_accessor (DataAccessor): The means of accessing data
        panel (Panel): The panel (ie genomic location, scale) we want
        datafile (str): Name of the bigwig data file
        data_range (tuple[int,int]): The range of values in the data file
    """
    item = panel.get_chrom(data_accessor).item_path(data_file)
    if panel.end - panel.start < 1000:
        (data, start, end) = get_bigwig(data_accessor, item, panel.start, panel.end)
    else:
        (data, start, end) = get_bigwig_stats(data_accessor, item, panel.start, panel.end)
    
    # clean & normalize input data range for eard (0..25)
    overflow_flag = []
    normalized_data = []
    scores = []
    x_x = []
    scale = 25/(data_range[1]-data_range[0])
    for i, x in enumerate(data):
        if x is None or math.isnan(x):
            x = 0
            overflow_flag.append(0)
            scores.append("{:.2f}".format(x))
        else:
            scores.append("{:.2f}".format(x))
            x = round((x-data_range[0])*scale)
            unbound_x = x
            x_x.append(x)
            x = max(0, min(25, x)) # 0-25
            overflow_flag.append(1 if x != unbound_x else 0) # unbounds
        normalized_data.append(x)
    return {
        "normalized_values": data_algorithm("NDZRL", bytearray(normalized_data)),
        "conservation_scores": data_algorithm("SZ", scores),
        "overflow_flag": data_algorithm("NDZRL", overflow_flag),
        "range": data_algorithm("NRL", [start, end])
    }


class GCWiggleDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope, accept: str) -> dict:
        """
        Handle a request for GC% wiggle data.

        Args:
            data_accessor (DataAccessor): The means of accessing data
            panel (Panel): The panel (ie genomic location, scale) we want
            scope: extra scope args (e.g. datafile name)

        Returns: A data dict (payload for Response object)
        """

        return get_wiggle_data(data_accessor, panel, "gc", (0,100))
    
class ComparaWiggleDataHandler(DataHandler):
    """
        Handle a request for Compara wiggle data (conservation scores).
        Signature as per GCDataHandler.process_data() above.
    """
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope, accept: str) -> dict:
        return get_wiggle_data(data_accessor, panel, self.get_datafile(scope), (-10,10))
