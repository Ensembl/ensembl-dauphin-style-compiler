import logging
from model.tracks import Tracks
from .begs import BegsFiles, ProgramInventory
from model.datamodel import DataModel
from model.datalocator import DataSourceResolver
from model.memcached import Memcached
from core.config import BOOT_TRACKS_TOML, MEMCACHED_PREFIX, MEMCACHED_BUMP_ON_RESTART
import toml, os.path

def all_boot_tracks():
    out = {}
    with open(BOOT_TRACKS_TOML) as f:
        toml_file = toml.loads(f.read())
        logging.warn(toml_file)
        for (version,file) in toml_file["versions"].items():
            path = os.path.join(os.path.dirname(BOOT_TRACKS_TOML),file)
            out[int(version)] = Tracks(path)
    return out

class DataAccessor:
    def __init__(self, version: int):
        self.reload(version)

    def reload(self, version: int):
        self.resolver : DataSourceResolver = DataSourceResolver(version)
        self.begs_files = BegsFiles()
        if version > 14:
            self.program_inventory = ProgramInventory(version)
        self.data_model = DataModel()
        self.cache = Memcached("{}:{}".format(MEMCACHED_PREFIX,version),MEMCACHED_BUMP_ON_RESTART)
        self.boot_tracks = all_boot_tracks()
        self.supported_versions = [16]

class DataAccessorCollection:
    def __init__(self):
        self._accessors = {}

    def get(self, version: int) -> DataAccessor:
        if version not in self._accessors:
            self._accessors[version] = DataAccessor(version)
        return self._accessors[version]
