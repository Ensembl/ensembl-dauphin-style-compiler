import logging
from model.tracks import Tracks
import toml
import os.path
from core.config import ASSETS_TOML, BOOT_TRACKS_TOML, METRIC_FILE, ASSETS_DIR
from typing import Any
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
            r = Response(0,{
                "boot": [channel,data_accessor.begs_files.boot_program(version)],  # should go when v14 is retired
                "hi": channel, # should go when v14 is retired
                "lo": channel, # should go when v14 is retired
                "namespace":  channel,
                "assets": load_assets(False),
                "chrome-assets": load_assets(True),
                "supports": data_accessor.begs_files.versions()
            })
            bundles = data_accessor.begs_files.all_bundles(version)
        except UnknownVersionException as e:
            return Response(1,"Backend out of date: Doesn't support egs version {}".format(e))
        for b in bundles:
            r.bundles.add(b)
        tracks_toml = toml.load(BOOT_TRACKS_TOML)
        r.add_tracks(Tracks(expanded_toml=tracks_toml))
        return r

class ProgramHandler(Handler):
    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any, metrics: ResponseMetrics, version: Version) -> Response:
        (want_channel, name) = payload
        if want_channel != channel:
            return Response(1,"Only know of programs in my own channel")
        try:
            bundle = data_accessor.begs_files.find_bundle(name,version)
        except UnknownVersionException as e:
            return Response(1,e)
        if bundle == None:
            return Response(1,"Unknown program {}".format(name))
        r = Response(2,[])
        r.bundles.add(bundle)
        return r
        
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
