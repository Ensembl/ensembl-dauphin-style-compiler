from model.tracks import Tracks
from .begs import BegsFiles, ProgramInventory
from model.datamodel import DataModel
from model.datalocator import DataSourceResolver
from model.memcached import Memcached
from core.config import BOOT_TRACKS_TOML, MEMCACHED_PREFIX, MEMCACHED_BUMP_ON_RESTART

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
        self.boot_tracks = Tracks(BOOT_TRACKS_TOML)
        self.supported_versions = [9,10,11,12,13,14,15]

class DataAccessorCollection:
    def __init__(self):
        self._accessors = {}

    def get(self, version: int) -> DataAccessor:
        if version not in self._accessors:
            self._accessors[version] = DataAccessor(version)
        return self._accessors[version]
