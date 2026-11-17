"""Fixed-scale, mean-coverage summaries for transcriptomic expression tracks."""

import math

from command.coremodel import DataHandler, DataAccessor, Panel
from data.v16.dataalgorithm import data_algorithm
from model.bigbed import get_bigwig_stats


def coverage_height(value: float | None) -> int:
    """Map coverage to 0..1000, with equally spaced 100/1000/10000 ticks."""
    if value is None or not math.isfinite(value) or value <= 0:
        return 0
    if value <= 100:
        bands = math.log10(value + 1) / math.log10(101)
    elif value <= 1000:
        bands = 1 + math.log10(value / 100)
    else:
        bands = 2 + math.log10(min(value, 10000) / 1000)
    return round(bands * 1000 / 3)


HIGH_COVERAGE = 20000


class CoverageDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope: dict, accept: str) -> dict:
        chrom = panel.get_chrom(data_accessor)
        item = chrom.item_filepath(self.get_datafile(scope))
        width = min(panel.end, chrom.size) - max(0, panel.start)
        if width <= 0:
            values, start, end = [], max(0, min(panel.start, chrom.size)), max(0, min(panel.start, chrom.size))
        else:
            values, start, end = get_bigwig_stats(
                data_accessor, item, panel.start, panel.end,
                consolidation="mean", nBins=min(500, width),
            )
        return {
            "values": data_algorithm("NDZRL", [coverage_height(value) for value in values]),
            "high": data_algorithm("NRL", [1 if value is not None and math.isfinite(value) and value > HIGH_COVERAGE else 0 for value in values]),
            "range": data_algorithm("NRL", [start, end]),
        }
