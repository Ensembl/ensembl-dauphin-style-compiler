from typing import Any
from .datasources import DataAccessor

class Response(object):
    def __init__(self, typ: int, payload: Any):
        self.typ = typ
        self.payload = payload
        self.bundles = set()

    def add_bundle(self, name: str):
        self.bundles.add(name)

class Handler:
    def process(self, data_accessor: DataAccessor, channel: Any,  payload: Any) -> Response:
        raise NotImplementedError("override process!")

class Panel(object):
    def __init__(self, data):
        (self.stick,self.scale,self.index) = data
        self.start = (1<<self.scale)*self.index
        self.end = (1<<self.scale)*(self.index+1)

class DataHandler:
    def process_data(self, data_accessor: DataAccessor, panel: Panel) -> Response:
        raise NotImplementedError("override process_data!")
