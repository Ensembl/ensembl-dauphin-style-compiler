from typing import Any, Dict
import logging
from .coremodel import Handler, Panel
from .response import Response
from .datasources import DataAccessor
from data.genedata import GeneDataHandler, GeneOverviewDataHandler, TranscriptDataHandler
from data.gc import WiggleDataHandler
from data.variant import VariantDataHandler
from data.sequence import ZoomedSeqDataHandler
from data.contig import ContigDataHandler, ShimmerContigDataHandler
from data.focusjump import FocusJumpHandler

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

    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any) -> Response:
        (channel,name,panel) = payload
        panel = Panel(panel)
        out = data_accessor.cache.get_data(channel,name,panel)
        if out != None:
            return out
        handler = self.handlers.get(name)
        if handler == None:
            return Response(1,"Unknown data endpoint {0}".format(name))
        out = handler.process_data(data_accessor, panel)
        data_accessor.cache.store_data(channel,name,panel,out)
        return out

class JumpHandler(Handler):
    def __init__(self):
        self.handlers = [
            FocusJumpHandler()
        ]

    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any) -> Response:
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
