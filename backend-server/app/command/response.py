from typing import Any, Dict
from command.eardo import EardoFile
from command.bundle import Bundle, BundleSet, EardoSet
from model.tracks import Tracks
import cbor2

class Response(object):
    def __init__(self, typ: int, payload: Any):
        self.payload = payload if typ == -1 else cbor2.dumps([typ,payload])
        self.bundles = BundleSet()
        self.eardos = EardoSet()
        self.tracks = Tracks()
        self.values = {}

    def add_eardo(self, eardo: EardoFile):
        self.eardos.add(eardo)

    def add_bundle(self, bundle: Bundle):
        self.bundles.add(bundle)

    def add_tracks(self, tracks: Tracks):
        self.tracks.merge(tracks)

    def add_values(self, namespace: str, column: str, values: Dict[str,str]):
        if namespace not in self.values:
            self.values[namespace] = {}
        if column not in self.values[namespace]:
            self.values[namespace][column] = values

    def len(self):
        return len(self.payload)
