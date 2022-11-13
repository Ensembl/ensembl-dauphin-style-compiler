from __future__ import annotations

import zlib, cbor2
from tokenize import Number
from typing import Any, ByteString, Dict, List, Optional, Union
import time
from .coremodel import Handler, Panel
from .response import Response
from .datasources import DataAccessor
from .exceptionres import DataException

from data.old.genedata8 import GeneDataHandler8, GeneOverviewDataHandler8, TranscriptDataHandler8, GeneLocationHandler8
from data.old.gc import WiggleDataHandler
from data.old.variant import VariantDataHandler
from data.old.sequence8 import ZoomedSeqDataHandler8
from data.old.contig import ContigDataHandler, ShimmerContigDataHandler

from data.v14.gene.genedata import GeneDataHandler, GeneOverviewDataHandler, TranscriptDataHandler
from data.v14.wiggle.gc import WiggleDataHandler2
from data.v14.variant import VariantDataHandler2
from data.v14.sequence import ZoomedSeqDataHandler
from data.v14.contig import ContigDataHandler2, ShimmerContigDataHandler2

from data.focusjump import FocusJumpHandler
from util.influx import ResponseMetrics
from model.version import Version

handlers = [
    ("gene-overview", GeneOverviewDataHandler8()),
    ("gene", GeneDataHandler8()),
    ("transcript", TranscriptDataHandler8(False)),
    ("zoomed-transcript", TranscriptDataHandler8(True)),
    ("zoomed-seq", ZoomedSeqDataHandler8()),
    ("gene-location", GeneLocationHandler8()),
    ("gc", WiggleDataHandler()),
    ("contig", ContigDataHandler()),
    ("shimmer-contig", ShimmerContigDataHandler()),
    ("variant", VariantDataHandler()),

    ("gene-overview", GeneOverviewDataHandler(),14),
    ("gene", GeneDataHandler(),14),
    ("transcript", TranscriptDataHandler(),14),
    ("zoomed-seq", ZoomedSeqDataHandler(),14),
    ("gc", WiggleDataHandler2(),14),
    ("contig", ContigDataHandler2(),14),
    ("shimmer-contig", ShimmerContigDataHandler2(),14),
    ("variant", VariantDataHandler2(),14),
]

def compress_payload(data):
    data = cbor2.dumps(data)
    return zlib.compress(data)

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
        if version.get_egs() < 14:
            (channel,name,panel,scope) = payload
            accept = ""
        else:
            (channel,name,panel,scope,accept) = payload
        panel = Panel(panel)
        out = data_accessor.cache.get_data([channel,name,panel.dumps(),scope,accept],version)
        if out != None:
            metrics.cache_hits += 1
            metrics.cache_hits_bytes += out.len()
            return out
        handler = self.get_handler(name,version)
        if handler == None:
            return Response(1,"Unknown data endpoint {0}".format(name))
        start = time.time()
        try:
            if version.get_egs() < 14:
                out = handler.process_data(data_accessor,panel,scope)
            else:
                data = handler.process_data(data_accessor,panel,scope,accept)
                data2 = {}
                if isinstance(data,list):
                    (data,data2) = tuple(data)
                invariant = data.pop('__invariant',False) or data2.pop('__invariant',False)
                out = Response(5,{'data': compress_payload(data), 'data2': compress_payload(data2), '__invariant': invariant })
        except DataException as e:
            out = e.to_response()
        time_taken_ms = (time.time() - start) * 1000.0
        metrics.runtime_num[(name,panel.scale)] += time_taken_ms
        metrics.runtime_denom[(name,panel.scale)] += 1
        metrics.cache_misses += 1
        metrics.cache_misses_bytes += out.len()
        data_accessor.cache.store_data([channel,name,panel.dumps(),scope,accept],version,out)
        return out

    def remote_prefix(self, payload: Any) -> Optional[List[str]]:
        return ["data",payload[1],payload[2][0]]

class JumpHandler(Handler):
    def __init__(self):
        self.handlers = [
            FocusJumpHandler()
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
