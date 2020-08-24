from typing import Any
from .datasources import DataAccessor

class Response(object):
    def __init__(self, typ: int, payload: Any):
        self.typ = typ
        self.payload = payload
        self.bundles = set()
        self.sticks = {}

    def add_bundle(self, name: str):
        self.bundles.add(name)

    def add_stick(self, name: str, data: Any):
        self.sticks[name] = data

class Handler:
    def process(self, data_accessor: DataAccessor, channel: Any,  payload: Any) -> Response:
        raise NotImplementedError("override process!")

