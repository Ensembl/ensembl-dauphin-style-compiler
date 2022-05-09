from typing import Any, Dict, List, Optional
import time
from .coremodel import Handler, Panel
from .response import Response
from .datasources import DataAccessor
from data.genedata import GeneDataHandler, GeneOverviewDataHandler, TranscriptDataHandler
from data.gc import WiggleDataHandler
from data.variant import VariantDataHandler
from data.sequence import ZoomedSeqDataHandler
from data.contig import ContigDataHandler, ShimmerContigDataHandler
from data.focusjump import FocusJumpHandler
from util.influx import ResponseMetrics
from model.version import Version

class DataHandler(Handler):
    def __init__(self):
        self.handlers : Dict[str,DataHandler] = {
            "zoomed-transcript": TranscriptDataHandler(True),
            "zoomed-seq": ZoomedSeqDataHandler(),
            "transcript": TranscriptDataHandler(False),
            "gene": GeneDataHandler(),
            "gene-overview": GeneOverviewDataHandler(),
            "gc": WiggleDataHandler(),
            "contig": ContigDataHandler(),
            "shimmer-contig": ShimmerContigDataHandler(),
            "variant": VariantDataHandler()
        }

    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any, metrics: ResponseMetrics, version: Version) -> Response:
        if len(payload) == 3:
            (channel,name,panel) = payload
            scope = []
        else:
            (channel,name,panel,scope) = payload
        panel = Panel(panel)
        out = data_accessor.cache.get_data(channel,name,panel)
        if out != None:
            metrics.cache_hits += 1
            metrics.cache_hits_bytes += out.len()
            return out
        handler = self.handlers.get(name)
        if handler == None:
            return Response(1,"Unknown data endpoint {0}".format(name))
        start = time.time()
        out = handler.process_data(data_accessor, panel)
        time_taken_ms = (time.time() - start) * 1000.0
        metrics.runtime_num[(name,panel.scale)] += time_taken_ms
        metrics.runtime_denom[(name,panel.scale)] += 1
        metrics.cache_misses += 1
        metrics.cache_misses_bytes += out.len()
        data_accessor.cache.store_data(channel,name,panel,out)
        return out

    def remote_prefix(self, payload: Any) -> Optional[List[str]]:
        return ["data",payload[1],payload[2][0]]

class JumpHandler(Handler):
    def __init__(self, data_accessor):
        self.handlers = [
            FocusJumpHandler(data_accessor)
        ]

    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any, metrics: ResponseMetrics, version: Version) -> Response:
        (location,) = payload
        for handler in self.handlers:
            jump = handler.get(data_accessor,location)
            if jump != None:
                return Response(6,{
                    "stick": jump[0],
                    "left": jump[1],
                    "right": jump[2]
                })
        return Response(6,{
            "no": True
        })

    def remote_prefix(self, payload: Any) -> Optional[List[str]]:
        return ["jump"]
