from typing import Any
from .coremodel import Response, Handler
from .datasources import DataAccessor

class ErrorHandler(Handler):
    def __init__(self, message: str):
        self.message = message

    def process(self, data_accessor: DataAccessor, channel: Any,  payload: Any) -> Response:
        return Response(1,self.message)

class BootstrapHandler(Handler):
    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any) -> Response:
        r = Response(0,[channel,data_accessor.begs_files.boot_program])
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
