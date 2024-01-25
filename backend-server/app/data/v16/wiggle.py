from command.coremodel import DataHandler, Panel, DataAccessor
from command.response import Response
from model.bigbed import get_bigwig_stats, get_bigwig
from .dataalgorithm import data_algorithm
import math

def get_wiggle_data(
        data_accessor: DataAccessor, panel: Panel, datafile: str, scale: float=1, shift: int=0
    ) -> dict:
    """
    Use the DataAccessor provided to access the wiggle data for a given location.

    Args:
        data_accessor (DataAccessor): The means of accessing data
        panel (Panel): The panel (ie genomic location, scale) we want
        datafile (str): Name of the bigwig data file
        scale, shift (float, int): scale/shift for data (to fit in 0..25)
    """
    item = panel.get_chrom(data_accessor).item_path(datafile)
    if panel.end - panel.start < 1000:
        (data, start, end) = get_bigwig(data_accessor, item, panel.start, panel.end)
    else:
        (data, start, end) = get_bigwig_stats(data_accessor, item, panel.start, panel.end)
    
    #input range: 0..100 (gc), ~-10..~10 (compara); output: 0..25
    data = [0 if x is None or math.isnan(x) else x for x in data]
    data = [round(x*scale)+shift for x in data]
    data = bytearray([max(0,min(25,x)) for x in data])

    return {
        "values": data_algorithm("NDZRL",data),
        "range": data_algorithm("NRL",[start, end])
    }


class GCWiggleDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope) -> dict:
        """
        Handle a request for GC% wiggle data.

        Args:
            data_accessor (DataAccessor): The means of accessing data
            panel (Panel): The panel (ie genomic location, scale) we want
            scope: extra scope args (e.g. datafile name)

        Returns: A data dict (payload for Response object)
        """

        return get_wiggle_data(data_accessor, panel, "gc", 0.25, 0)
    
class ComparaWiggleDataHandler(DataHandler):
    """
        Handle a request for Compara wiggle data (conservation scores).
        Signature as per GCDataHandler.process_data() above.
    """
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope) -> dict:
        return get_wiggle_data(data_accessor, panel, self.get_datafile(scope), 5, 10)
