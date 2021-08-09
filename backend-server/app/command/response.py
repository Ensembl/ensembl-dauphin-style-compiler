from typing import Any
import cbor2

class Response(object):
    def __init__(self, typ: int, payload: Any):
        self.payload = payload if typ == -1 else cbor2.dumps([typ,payload])
        self.bundles = set()

    def add_bundle(self, name: str):
        self.bundles.add(name)
