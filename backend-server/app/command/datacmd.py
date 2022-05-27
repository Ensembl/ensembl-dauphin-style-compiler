from __future__ import annotations

import logging
from tokenize import Number
from typing import Any, Dict, List, Optional
import time
from .coremodel import Handler, Panel
from .response import Response
from .datasources import DataAccessor
from data.genedata import GeneDataHandler, GeneOverviewDataHandler, TranscriptDataHandler
from data.genedata8 import GeneDataHandler8, GeneLocationHandler8, GeneOverviewDataHandler8, TranscriptDataHandler8
from data.gc import WiggleDataHandler
from data.variant import VariantDataHandler
from data.sequence import ZoomedSeqDataHandler
from data.sequence8 import ZoomedSeqDataHandler8
from data.contig import ContigDataHandler, ShimmerContigDataHandler
from data.focusjump import FocusJumpHandler
from util.influx import ResponseMetrics
from model.version import Version

handlers = [
    ("gene-overview", GeneOverviewDataHandler()),
    ("gene", GeneDataHandler()),
    ("transcript", TranscriptDataHandler(False)),
    ("zoomed-transcript", TranscriptDataHandler(True)),
    ("zoomed-seq", ZoomedSeqDataHandler()),

    ("gene-location", GeneLocationHandler8(), 8),
    ("gene-overview", GeneOverviewDataHandler8(), 8),
    ("gene", GeneDataHandler8(), 8),
    ("transcript", TranscriptDataHandler8(False), 8),
    ("zoomed-transcript", TranscriptDataHandler8(True), 8),
    ("zoomed-seq", ZoomedSeqDataHandler8(), 8),

    ("gc", WiggleDataHandler()),
    ("contig", ContigDataHandler()),
    ("shimmer-contig", ShimmerContigDataHandler()),
    ("variant", VariantDataHandler())
]

def make_handlers_for_version(version_wanted: Number) -> Dict[str,DataHandler]:
    out = {}
    versions = {}
    for line in handlers:
        # Add dummy "version zero" where version not specified and then extract
        if len(line) == 2:
            line = (line[0],line[1],0)
        (name,handler,version) = line
        old_version = versions.get(name,-1)
        # If not too new for what we're building and newere than anything so far, record it.
        if version <= version_wanted and version > old_version:
            out[name] = handler
            versions[name] = version
    return out

class DataHandler(Handler):
    def __init__(self):
        max_minver = max( x[2] if len(x)>2 else 0 for x in handlers ) # Largest minimum version specified in any handler
        self.handlers : List[Dict[str,DataHandler]] = [ 
            make_handlers_for_version(i) for i in range(max_minver+1)
        ]

    def get_handler(self, name: str, version: Number) -> DataHandler:
        return self.handlers[min(version.get_egs(),len(self.handlers)-1)].get(name,None)

    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any, metrics: ResponseMetrics, version: Version) -> Response:
        scope = []
        if len(payload) == 3:
            (channel,name,panel) = payload
            scope = []
        else:
            (channel,name,panel,new_scope) = payload
            scope = new_scope
        panel = Panel(panel)
        out = data_accessor.cache.get_data(channel,name,version,panel,scope)
        if out != None:
            metrics.cache_hits += 1
            metrics.cache_hits_bytes += out.len()
            return out
        handler = self.get_handler(name,version)
        if handler == None:
            return Response(1,"Unknown data endpoint {0}".format(name))
        start = time.time()
        out = handler.process_data(data_accessor, panel, scope)
        time_taken_ms = (time.time() - start) * 1000.0
        metrics.runtime_num[(name,panel.scale)] += time_taken_ms
        metrics.runtime_denom[(name,panel.scale)] += 1
        metrics.cache_misses += 1
        metrics.cache_misses_bytes += out.len()
        data_accessor.cache.store_data(channel,name,version,panel,scope,out)
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
            jump = handler.get(data_accessor,location,version)
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
