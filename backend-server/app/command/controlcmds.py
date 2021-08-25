from core.config import LO_PORT, METRIC_FILE
from typing import Any
from .coremodel import Handler
from .response import Response
from .datasources import DataAccessor
import datetime
import logging
from urllib.parse import urlparse

class ErrorHandler(Handler):
    def __init__(self, message: str):
        self.message = message

    def process(self, data_accessor: DataAccessor, channel: Any,  payload: Any) -> Response:
        return Response(1,self.message)

def lo_port(channel):
    out = list(channel)
    if channel[0] == 0: # URL type
        url = urlparse(channel[1])
        netloc = url.netloc
        hostport = netloc.split(':')
        if len(hostport) > 1:
            netloc = hostport[0]+':'+str(int(hostport[1])+1)
        url = url._replace(netloc=netloc)
        out[1] = url.geturl()
    return out

class BootstrapHandler(Handler):
    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any) -> Response:
        lo_channel = (lo_port(channel) if LO_PORT else channel)
        r = Response(0,{
            "boot": [channel,data_accessor.begs_files.boot_program],
            "hi":  channel,
            "lo":  lo_channel,
        })
        for b in data_accessor.begs_files.all_bundles():
            r.bundles.add(b)
        return r

class ProgramHandler(Handler):
    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any) -> Response:
        (want_channel, name) = payload
        if want_channel != channel:
            return Response(1,"Only know of programs in my own channel")
        bundle = data_accessor.begs_files.find_bundle(name)
        if bundle == None:
            return Response(1,"Unknown program {}".format(name))
        r = Response(2,[])
        r.add_bundle(bundle)
        return r

class StickHandler(Handler):
    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any) -> Response:
        (stick_name,) = payload
        chromosome = data_accessor.data_model.stick(data_accessor,stick_name)
        if chromosome == None:
            return Response(1,"Unknown stick {0}".format(stick_name))
        else:
            return Response(3,{
                "id": stick_name,
                "size": chromosome.size,
                "topology": 0 if chromosome.topology == "linear" else 1,
                "tags": [t for t in chromosome.tags]
            })

class StickAuthorityHandler(Handler):
    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any) -> Response:
        sa_start_prog = data_accessor.begs_files.stickauthority_startup_program
        sa_lookup_prog = data_accessor.begs_files.stickauthority_lookup_program
        sa_jump_prog = data_accessor.begs_files.stickauthority_jump_program
        if sa_start_prog != None:
            r = Response(4,[channel,sa_start_prog,sa_lookup_prog,sa_jump_prog])
        else:
            return Response(1,"I am not an authority")
        return r

class FailureHandler(Handler):
    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any) -> Response:
        (identity,text,major,minor) = payload
        now = datetime.datetime.now().replace(microsecond=0).isoformat()
        with open(METRIC_FILE,"a") as f:
            parts = [now,hex(identity),major,hex(minor),text]
            f.write("\t".join([str(x) for x in parts])+"\n")
        r = Response(2,[])
        return r
