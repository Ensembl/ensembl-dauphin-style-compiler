import logging
import json
from typing import Any
from .datasources import DataAccessor
from .begs import Bundle
from .coremodel import Response, Handler
from .controlcmds import BootstrapHandler, ProgramHandler, ErrorHandler, StickHandler, StickAuthorityHandler
from .datacmd import DataHandler

data_accessor = DataAccessor()        

def type_to_handler(typ: int) -> Any:
    if typ == 0:
        return BootstrapHandler()
    elif typ == 1:
        return ProgramHandler()
    elif typ == 2:
        return StickHandler()
    elif typ == 3:
        return StickAuthorityHandler()
    elif typ == 4:
        return DataHandler()
    else:
        return ErrorHandler("unsupported command type ({0})".format(typ))

def process_request(channel: Any, typ: int,payload: Any):
    handler = type_to_handler(typ)
    return handler.process(data_accessor,channel,payload)

def process_packet(packet_cbor: Any) -> Any:
    channel = packet_cbor["channel"]
    response = []
    bundles = set()
    for p in packet_cbor["requests"]:
        (msgid,typ,payload) = p
        r = process_request(channel,typ,payload)
        response.append([msgid,r.typ,r.payload])
        bundles |= r.bundles
    begs_files = data_accessor.begs_files
    return {
        "responses": response,
        "programs": [ begs_files.add_bundle(x) for x in bundles ]
    }
