from .begs import BegsFiles
from model.datamodel import DataModel
from model.datalocator import DataSourceResolver
from model.memcached import Memcached
from core.config import MEMCACHED_PREFIX, MEMCACHED_BUMP_ON_RESTART

class DataAccessor:
    def __init__(self, version: int):
        self.reload(version)

    def reload(self, version: int):
        self.resolver : DataSourceResolver = DataSourceResolver(version)
        self.begs_files = BegsFiles()
        self.data_model = DataModel()
        self.cache = Memcached(MEMCACHED_PREFIX,MEMCACHED_BUMP_ON_RESTART)

class DataAccessorCollection:
    def __init__(self):
        self._accessors = {}

    def get(self, version: int) -> DataAccessor:
        if version not in self._accessors:
            self._accessors[version] = DataAccessor(version)
        return self._accessors[version]
