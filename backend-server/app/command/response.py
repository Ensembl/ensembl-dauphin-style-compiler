from typing import Any
from model.tracks import Tracks
import cbor2

class Response(object):
    def __init__(self, typ: int, payload: Any):
        self.payload = payload if typ == -1 else cbor2.dumps([typ,payload])
        self.bundles = set()
        self.tracks = Tracks()

    def add_bundle(self, name: str):
        self.bundles.add(name)

    def add_tracks(self, tracks: Tracks):
        self.tracks.merge(tracks)

    def len(self):
        return len(self.payload)
