import logging
import json
from typing import Any, Tuple
from .datasources import DataAccessor
from .begs import Bundle
from .coremodel import Handler
from .response import Response
from .controlcmds import BootstrapHandler, ProgramHandler, ErrorHandler, StickHandler, StickAuthorityHandler, FailureHandler
from .datacmd import DataHandler, JumpHandler

data_accessor = DataAccessor()        

handlers = {
    0: BootstrapHandler(),
    1: ProgramHandler(),
    2: StickHandler(),
    3: StickAuthorityHandler(),
    4: DataHandler(),
    5: JumpHandler(data_accessor),
    6: FailureHandler()
}

def type_to_handler(typ: int) -> Any:
    handler = handlers.get(typ)
    if handler == None:
        return ErrorHandler("unsupported command type ({0})".format(typ))
    return handler

def process_request(channel: Tuple[int,str], typ: int, payload: Any):
    handler = type_to_handler(typ)
    return handler.process(data_accessor,channel,payload)

def process_packet(packet_cbor: Any, high_priority: bool) -> Any:
    channel = packet_cbor["channel"]
    response = []
    bundles = set()
    for p in packet_cbor["requests"]:
        (msgid,typ,payload) = p
        r = process_request(channel,typ,payload)
        response.append([msgid,r.payload])
        bundles |= r.bundles
    begs_files = data_accessor.begs_files
    return (response,[ begs_files.add_bundle(x) for x in bundles ])
