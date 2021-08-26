"""
See the NOTICE file distributed with this work for additional information
regarding copyright ownership.


Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at
http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
"""

import logging

from typing import Any
from enum import Enum
from fastapi import APIRouter, HTTPException
from loguru import logger
from starlette import responses
from starlette.datastructures import Headers
from starlette.status import HTTP_404_NOT_FOUND
from starlette.requests import Request
from starlette.responses import Response
import cbor2
from cbor2 import CBOREncoder
from command.packet import process_packet
from io import BytesIO

router = APIRouter()

# Some of our cbor is from caches and already serialised so we have to build our response ourselves
def build_response(responses,programs) -> Any:
    with BytesIO() as fp:
        encoder = CBOREncoder(fp)
        encoder.encode_length(5,2)
        encoder.encode("responses")
        encoder.encode_length(4,len(responses))
        for (id,payload) in responses:
            encoder.encode_length(4,2)
            encoder.encode(id)
            encoder.fp.write(payload) # this line is the key swerve
        encoder.encode("programs")
        encoder.encode(programs)
        return fp.getvalue()

class PacketPriority(str, Enum):
    hi = "hi"
    lo = "lo"

@router.post("/{priority}", name="data")
async def data(priority: PacketPriority, request: Request):
    """
    Data endpoint for peregrine-web. Priorities are ignored and exist to
    allow deployment routers to partition small, low latency packets from larger
    packets with fewer latency requeiments to route them to different resources.
    """
    headers = Headers()
    body = b''
    async for chunk in request.stream():
        body += chunk
    input_data = cbor2.loads(body)
    (responses,programs) = process_packet(input_data,priority=="hi")
    body = build_response(responses,programs)
    return Response(content=body,media_type="application/cbor")
