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
from starlette.requests import Request
from starlette.responses import Response
import cbor2
from cbor2 import CBOREncoder
from command.packet import process_packet
from command.packetvalidation import is_valid_channel, is_valid_command_payload
from core.exceptions import UpstreamRequestError
from io import BytesIO

router = APIRouter()

# Some of our cbor is from caches and already serialised so we have to build our response ourselves
def build_response(responses,programs,eardos,channel,tracks) -> Any:
    num_items = 4
    if tracks is not None:
        num_items += 1
    with BytesIO() as fp:
        encoder = CBOREncoder(fp)
        encoder.encode_length(5,num_items)
        encoder.encode("responses")
        encoder.encode_length(4,len(responses))
        for (id,payload) in responses:
            encoder.encode_length(4,2)
            encoder.encode(id)
            encoder.fp.write(payload) # this line is the key swerve
        encoder.encode("programs")
        encoder.encode(programs)
        if tracks is not None:
            encoder.encode("tracks-packed")
            encoder.encode(tracks)
        encoder.encode("eardos")
        encoder.encode(eardos)
        encoder.encode("channel")
        encoder.encode(channel)
        return fp.getvalue()

class PacketPriority(str, Enum):
    hi = "hi"
    lo = "lo"


def decode_packet(body: bytes) -> dict:
    """Decode one CBOR packet and validate its envelope before dispatch."""
    stream = BytesIO(body)
    try:
        packet = cbor2.load(stream)
    except (cbor2.CBORDecodeError, ValueError, TypeError, ArithmeticError, RecursionError) as exc:
        raise HTTPException(status_code=400, detail="Invalid CBOR body") from exc

    if not isinstance(packet, dict):
        raise HTTPException(status_code=400, detail="Expected a CBOR map")
    if stream.read(1):
        raise HTTPException(status_code=400, detail="Unexpected data after CBOR packet")
    if "channel" not in packet or "requests" not in packet:
        raise HTTPException(status_code=400, detail="Missing channel or requests")

    if not is_valid_channel(packet["channel"]):
        raise HTTPException(status_code=400, detail="Invalid channel")

    requests = packet["requests"]
    if not isinstance(requests, (list, tuple)):
        raise HTTPException(status_code=400, detail="Expected a requests array")
    for request in requests:
        if (
            not isinstance(request, (list, tuple))
            or len(request) != 3
            or any(type(value) is not int or value < 0 for value in request[:2])
        ):
            raise HTTPException(
                status_code=400,
                detail="Each request must contain a non-negative integer ID, command type and payload",
            )
        if not is_valid_command_payload(request[1], request[2]):
            raise HTTPException(
                status_code=400,
                detail=f"Invalid payload for command {request[1]}",
            )

    version = packet.get("version")
    if version is not None and (
        not isinstance(version, dict)
        or type(version.get("egs", 0)) is not int
        or version.get("egs", 0) < 0
    ):
        raise HTTPException(status_code=400, detail="Invalid version")
    return packet


@router.post("/{priority}", name="data")
async def data(priority: PacketPriority, request: Request):
    """
    Data endpoint for peregrine-web. Priorities are ignored and exist to
    allow deployment routers to partition small, low latency packets from larger
    packets with fewer latency requeiments to route them to different resources.
    """
    body = b''
    async for chunk in request.stream():
        body += chunk
    try:
        input_data = decode_packet(body)
    except HTTPException as exc:
        decoder_error = exc.__cause__
        logging.warning(
            "Rejected data packet: reason=%r decoder_error_type=%s decoder_error=%r "
            "priority=%s bytes=%d stamp=%r request_id=%r referer=%r",
            exc.detail,
            type(decoder_error).__name__ if decoder_error is not None else None,
            str(decoder_error)[:512] if decoder_error is not None else None,
            priority.value,
            len(body),
            request.query_params.get("stamp", "")[:128],
            request.headers.get("x-request-id", "")[:128],
            request.headers.get("referer", "")[:512],
        )
        raise
    try:
        (responses,programs,eardos,channel,tracks) = process_packet(input_data,priority=="hi")
    except UpstreamRequestError as exc:
        logging.warning(
            "Upstream request failed: service=%r method=%s upstream_url=%r upstream_status=%r "
            "detail=%r response_status=%d incoming_url=%r stamp=%r request_id=%r referer=%r",
            exc.service, exc.method, exc.url[:2048], exc.upstream_status, exc.detail,
            exc.status_code, str(request.url)[:2048],
            request.query_params.get("stamp", "")[:128],
            request.headers.get("x-request-id", "")[:128],
            request.headers.get("referer", "")[:1024],
        )
        raise HTTPException(status_code=exc.status_code, detail=f"{exc.service} request failed") from None
    body = build_response(responses,programs,eardos,channel,tracks)
    return Response(content=body,media_type="application/cbor")
