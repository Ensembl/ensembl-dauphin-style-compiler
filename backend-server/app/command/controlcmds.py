import logging
from model.tracks import Tracks
from typing import List
import toml
import os.path
from core.config import ASSETS_TOML, BOOT_TRACKS_TOML, METRIC_FILE, ASSETS_DIR
from typing import Any, Callable, Optional
from .coremodel import Handler
from .response import Response
from .datasources import DataAccessor
import datetime
from urllib.parse import urlparse
from util.influx import ResponseMetrics
from model.version import Version
from command.begs import UnknownVersionException

class ErrorHandler(Handler):
    def __init__(self, message: str):
        self.message = message

    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any, metrics: ResponseMetrics) -> Response:
        return Response(1,self.message)

def load_assets(chrome: bool):
    assets = {}
    toml_data = toml.load(ASSETS_TOML)
    toml_data.get('sources',{})
    for (name,data) in toml_data.get('sources',{}).items():
        asset = dict(data)
        is_chrome = asset.get("chrome",False)
        if is_chrome == chrome:
            with open(os.path.join(ASSETS_DIR,data["file"]),"rb") as f:
                asset["data"] = f.read()
            assets[name] = asset
    return assets

class BootstrapHandler(Handler):
    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any, metrics: ResponseMetrics, version: Version) -> Response:
        try:
            if version.get_egs() < 15:
                r = Response(0,{
                    "boot": [channel,data_accessor.begs_files.boot_program(version)],
                    "hi": channel,
                    "lo": channel,
                    "namespace":  channel,
                    "assets": load_assets(False),
                    "chrome-assets": load_assets(True),
                    "supports": data_accessor.supported_versions
                })
                bundles = data_accessor.begs_files.boot_bundles(version)
            else:
                r = Response(0,{
                    "namespace":  channel,
                    "assets": load_assets(False),
                    "chrome-assets": load_assets(True),
                    "supports": data_accessor.supported_versions
                })
                bundles = data_accessor.program_inventory.boot_bundles(version.get_egs())
        except UnknownVersionException as e:
            return Response(1,"Backend out of date: Doesn't support egs version {}".format(e))
        for b in bundles:
            r.add_bundle(b)
        r.add_tracks(data_accessor.boot_tracks)
        return r

    def remote_prefix(self, payload: Any) -> Optional[List[str]]:
        return ["boot"]

class ProgramHandler(Handler):
    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any, metrics: ResponseMetrics, version: Version) -> Response:
        logging.warn("ProgramHandler {}".format(payload))
        (prog_set,name,prog_version) = payload
        try:
            if version.get_egs() < 15:
                bundle = data_accessor.begs_files.find_bundle(name,version)
            else:
                bundle = data_accessor.program_inventory.find_bundle(prog_set,name,prog_version)
        except UnknownVersionException as e:
            return Response(1,e)
        r = Response(2,[])
        if bundle != None:
            r.add_bundle(bundle)
        return r

    def remote_prefix(self, payload: Any) -> Optional[List[str]]:
        return ["program"]

class StickHandler(Handler):
    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any, metrics: ResponseMetrics, version: Version) -> Response:
        (stick_name,) = payload
        chromosome = data_accessor.data_model.try_stick(data_accessor,stick_name)
        if chromosome == None:
            return Response(3,{
                "error": "Unknown stick {0}".format(stick_name)
            })
        else:
            return Response(3,{
                "id": stick_name,
                "size": chromosome.size,
                "topology": 0 if chromosome.topology == "linear" else 1,
                "tags": [t for t in chromosome.tags]
            })

# doesn't exist v15 onwards
class StickAuthorityHandler(Handler):
    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any, metrics: ResponseMetrics, version: Version) -> Response:
        try:
            sa_start_prog = data_accessor.begs_files.authority_startup_program(version)
            sa_lookup_prog = data_accessor.begs_files.authority_lookup_program(version)
            sa_jump_prog = data_accessor.begs_files.authority_jump_program(version)
            return Response(4,[channel,sa_start_prog,sa_lookup_prog,sa_jump_prog])
        except UnknownVersionException as e:
            return Response(1,e)

class ExpansionHandler(Handler):
    def __init__(self, expansions) -> None:
        super().__init__()
        self._expansions_obj = expansions
        self._expansions = None

    def _get(self, data_accessor: DataAccessor, channel, name):
        expansion = data_accessor.boot_tracks.get_expansion(name)
        if expansion is not None:
            return getattr(self._expansions_obj,expansion.callback())
        else:
            return None

    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any, metrics: ResponseMetrics, version: Version) -> Response:
        try:
            r = Response(7,[])
            (name,step) = payload
            callable = self._get(data_accessor,channel,name)
            if callable is not None:
                tracks = callable(step)
            if tracks is not None:
                r.add_tracks(tracks)
            return r
        except UnknownVersionException as e:
            return Response(1,e)
