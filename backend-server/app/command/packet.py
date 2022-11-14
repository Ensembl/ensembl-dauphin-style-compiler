import collections
import imp
import logging

from command.bundle import BundleSet
from model.expansions import Expansions
from command.response import Response
from command.coremodel import Handler
from model.tracks import Tracks
import cbor2
import urllib
from typing import Any, List, Tuple
from .datasources import DataAccessor, DataAccessorCollection
from .controlcmds import BootstrapHandler, ProgramHandler, ErrorHandler, StickHandler, StickAuthorityHandler, ExpansionHandler
from .metriccmd import MetricHandler
from .datacmd import DataHandler, JumpHandler
from util.influx import ResponseMetrics
from model.version import Version
from core.config import DEFAULT_CHANNEL

data_accessor_collection = DataAccessorCollection()        

expansions = Expansions()

handlers = {
    0: BootstrapHandler(),
    1: ProgramHandler(),
    2: StickHandler(),
    3: StickAuthorityHandler(), # doesn't exist v15 onwards
    4: DataHandler(),
    5: JumpHandler(),
    6: MetricHandler(),
    7: ExpansionHandler(expansions)
}

def type_to_handler(typ: int) -> Handler:
    handler = handlers.get(typ)
    if handler == None:
        return ErrorHandler("unsupported command type ({0})".format(typ))
    return handler

def do_request_remote(url,channel,messages, high_priority: bool, version: Version):
    suffix = "hi" if high_priority else "lo"
    request = cbor2.dumps({
        "channel": channel,
        "requests": messages,
        "version": version.encode()
    })
    upstream = url + "/" + suffix
    logging.debug("delegating to {0}".format(upstream))
    with urllib.request.urlopen(upstream,data=request) as response:
        return cbor2.loads(response.read())

def extract_remote_request(data_accessor: DataAccessor, typ: int, payload: Any):
    handler = type_to_handler(typ)
    prefix = handler.remote_prefix(payload)
    if prefix != None:
        override = data_accessor.resolver.find_override(prefix)
        if override != None:
            return override
    return None

def process_local_request(data_accessor: DataAccessor,channel: Tuple[int,str], typ: int, payload: Any, metrics: ResponseMetrics, version: Version) -> Response:
    handler = type_to_handler(typ)
    return handler.process(data_accessor,channel,payload,metrics,version)

def replace_empty_channel(channel: Tuple[str,str]) -> Tuple[str,str]:
    if channel[0] == "" and channel[1] == "":
        channel = DEFAULT_CHANNEL
    return channel

def process_packet(packet_cbor: Any, high_priority: bool) -> Any:
    metrics = ResponseMetrics("realtime" if high_priority else "batch")
    channel = replace_empty_channel(packet_cbor["channel"])
    response = []
    program_data = []
    local_requests = []
    remote_requests = collections.defaultdict(list)
    version = Version(packet_cbor.get("version",None))
    data_accessor = data_accessor_collection.get(version.get_egs())
    bundles = BundleSet()
    tracks = Tracks()
    # separate into local and remote
    metrics.count_packets += len(packet_cbor["requests"])
    for p in packet_cbor["requests"]:
        (msgid,typ,payload) = p
        override = extract_remote_request(data_accessor,typ,payload)
        if override != None:
            remote_requests[override].append(p)
        else:
            local_requests.append((msgid,typ,payload))
    # remote stuff
    remote_tracks = []
    for (request,messages) in remote_requests.items():
        r = do_request_remote(request,channel,messages,high_priority,version)
        response += [[x[0],cbor2.dumps(x[1])] for x in r["responses"]]
        program_data += r["programs"]
        if "tracks-packed" in r and len(r["tracks-packed"])>0:
            tracks.add_cookeds(r["tracks-packed"])
    # local stuff
    for (msgid,typ,payload) in local_requests:
        if version.get_egs() in data_accessor.supported_versions:
            r = process_local_request(data_accessor,channel,typ,payload,metrics,version)
            response.append([msgid,r.payload])
            bundles.merge(r.bundles)
            tracks.merge(r.tracks)
        else:
            response.append([msgid,Response(8,[0]).payload])
    metrics.send()
    tracks = tracks.dump_for_wire()
    return (response,bundles.bundle_data(),channel,tracks)
