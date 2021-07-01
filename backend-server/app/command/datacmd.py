from typing import Any, Dict
import logging
from .coremodel import Handler, Panel, Response
from .datasources import DataAccessor
from data.genedata import GeneDataHandler, GeneOverviewDataHandler, TranscriptDataHandler
from data.wiggle import WiggleDataHandler

class DataHandler(Handler):
    def __init__(self):
        self.handlers : Dict[str,DataHandler] = {
            "zoomed-transcript": TranscriptDataHandler(True),
            "transcript": TranscriptDataHandler(False),
            "gene": GeneDataHandler(),
            "gene-overview": GeneOverviewDataHandler(),
            "gc": WiggleDataHandler(),
        }

    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any) -> Response:
        (_channel,name,panel) = payload
        panel = Panel(panel)
        handler = self.handlers.get(name)
        if handler == None:
            return Response(1,"Unknown data endpoint {0}".format(name))
        return handler.process_data(data_accessor, panel)
