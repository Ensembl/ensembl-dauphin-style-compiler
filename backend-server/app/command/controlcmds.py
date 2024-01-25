from core.config import ASSETS_TOML, ASSETS_DIR
from .coremodel import Handler
from .response import Response
from .datasources import DataAccessor
from util.influx import ResponseMetrics
from model.version import Version
from command.begs import UnknownVersionException
import os.path
import toml

class ErrorHandler(Handler):
    def __init__(self, message: str):
        self.message = message

    def process(self, data_accessor: DataAccessor, channel, payload, metrics: ResponseMetrics, version: Version) -> Response:
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
    def process(self, data_accessor: DataAccessor, channel, payload, metrics: ResponseMetrics, version: Version) -> Response:
        try:
            r = Response(0,{
                "namespace":  channel,
                "assets": load_assets(False),
                "chrome-assets": load_assets(True),
                "supports": data_accessor.supported_versions
            })
            eardos = data_accessor.program_inventory.boot_eardos(version.get_egs())
        except UnknownVersionException as e:
            return Response(1,"Backend out of date: Doesn't support egs version {}".format(e))
        for e in eardos:
            r.add_eardo(e)
        r.add_tracks(data_accessor.boot_tracks[version.get_egs()])
        return r

    def remote_prefix(self, payload) -> list[str]:
        return ["boot"]

class ProgramHandler(Handler):
    def process(self, data_accessor: DataAccessor, channel, payload, metrics: ResponseMetrics, version: Version) -> Response:
        (prog_set,name,prog_version) = payload
        eardo = []
        try:
            eardo = data_accessor.program_inventory.find_eardo_bundle(prog_set,name,prog_version)
        except UnknownVersionException as e:
            return Response(1,e)
        r = Response(2,[])
        if eardo != None:
            r.add_eardo(eardo)
        return r

    def remote_prefix(self, payload) -> list[str]:
        return ["program"]

class StickHandler(Handler):
    def process(self, data_accessor: DataAccessor, channel, payload, metrics: ResponseMetrics, version: Version) -> Response:
        (stick_name,) = payload
        chromosome = data_accessor.data_model.stick(data_accessor,stick_name)
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

class ExpansionHandler(Handler):
    def __init__(self, expansions) -> None:
        super().__init__()
        self._expansions_obj = expansions
        self._expansions = None

    def _get(self, data_accessor: DataAccessor, channel, name, version):
        expansion = data_accessor.boot_tracks[version.get_egs()].get_expansion(name)
        if expansion is not None:
            return getattr(self._expansions_obj,expansion.callback())
        else:
            return None

    def process(self, data_accessor: DataAccessor, channel, payload, metrics: ResponseMetrics, version: Version) -> Response:
        try:
            r = Response(7,[])
            (name,step) = payload
            callable = self._get(data_accessor,channel,name,version)
            if callable is not None:
                tracks = callable(step)
            if tracks is not None:
                r.add_tracks(tracks)
            return r
        except UnknownVersionException as e:
            return Response(1,e)
