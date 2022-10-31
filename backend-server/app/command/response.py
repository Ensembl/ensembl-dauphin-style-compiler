from typing import Any
from command.begs import Bundle, BundleSet
from model.tracks import Tracks
import cbor2

class Response(object):
    def __init__(self, typ: int, payload: Any):
        self.payload = payload if typ == -1 else cbor2.dumps([typ,payload])
        self.bundles = BundleSet()
        self.tracks = Tracks()

    def add_bundle(self, bundle: Bundle):
        self.bundles.add(bundle)

    def add_tracks(self, tracks: Tracks):
        self.tracks.merge(tracks)

    def len(self):
        return len(self.payload)
