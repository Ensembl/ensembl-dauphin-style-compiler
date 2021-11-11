import collections
import logging
import cbor2
import urllib
from typing import Any, List, Tuple
from .datasources import DataAccessor
from .begs import Bundle
from .coremodel import Handler
from .response import Response
from .controlcmds import BootstrapHandler, ProgramHandler, ErrorHandler, StickHandler, StickAuthorityHandler
from .metriccmd import MetricHandler
from .datacmd import DataHandler, JumpHandler
from util.influx import ResponseMetrics
from model.version import Version

data_accessor = DataAccessor()        

handlers = {
    0: BootstrapHandler(),
    1: ProgramHandler(),
    2: StickHandler(),
    3: StickAuthorityHandler(),
    4: DataHandler(),
    5: JumpHandler(data_accessor),
    6: MetricHandler()
}

def type_to_handler(typ: int) -> Any:
    handler = handlers.get(typ)
    if handler == None:
        return ErrorHandler("unsupported command type ({0})".format(typ))
    return handler

def do_request_remote(url,messages, high_priority: bool, version: Version):
    suffix = "hi" if high_priority else "lo"
    request = cbor2.dumps({
        "channel": [0,url],
        "requests": messages,
        "version": version.encode()
    })
    upstream = url + "/" + suffix
    logging.debug("delegating to {0}".format(upstream))
    with urllib.request.urlopen(upstream,data=request) as response:
        return cbor2.loads(response.read())

def extract_remote_request(channel: Tuple[int,str], typ: int, payload: Any):
    handler = type_to_handler(typ)
    prefix = handler.remote_prefix(payload)
    if prefix != None:
        override = data_accessor.resolver.find_override(prefix)
        if override != None:
            return override
    return None

def process_local_request(channel: Tuple[int,str], typ: int, payload: Any, metrics: ResponseMetrics, version: Version):
    handler = type_to_handler(typ)
    return handler.process(data_accessor,channel,payload,metrics,version)

def process_packet(packet_cbor: Any, high_priority: bool) -> Any:
    metrics = ResponseMetrics("realtime" if high_priority else "batch")
    channel = packet_cbor["channel"]
    response = []
    bundles = set()
    local_requests = []
    remote_requests = collections.defaultdict(list)
    version = Version(packet_cbor.get("version",None))
    # anything that should be remote
    metrics.count_packets += len(packet_cbor["requests"])
    for p in packet_cbor["requests"]:
        (msgid,typ,payload) = p
        override = extract_remote_request(channel,typ,payload)
        if override != None:
            remote_requests[override].append(p)
        else:
            local_requests.append((msgid,typ,payload))
    for (request,messages) in remote_requests.items():
        r = do_request_remote(request,messages,high_priority,version)
        response += [[x[0],cbor2.dumps(x[1])] for x in r["responses"]]
        bundles |= set(r["programs"])
    # local stuff
    for (msgid,typ,payload) in local_requests:
        r = process_local_request(channel,typ,payload,metrics,version)
        response.append([msgid,r.payload])
        bundles |= r.bundles
    begs_files = data_accessor.begs_files
    metrics.send()
    return (response,[ begs_files.add_bundle(x,version) for x in bundles ])
