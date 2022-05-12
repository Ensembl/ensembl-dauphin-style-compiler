from .begs import BegsFiles
from model.datamodel import DataModel
from model.datalocator import DataSourceResolver
from model.memcached import Memcached
from core.config import MEMCACHED_PREFIX, MEMCACHED_BUMP_ON_RESTART

class DataAccessor:
    def __init__(self):
        self.reload()

    def reload(self):
        self.resolver : DataSourceResolver = DataSourceResolver()
        self.begs_files = BegsFiles()
        self.data_model = DataModel(self.resolver)
        self.cache = Memcached(MEMCACHED_PREFIX,MEMCACHED_BUMP_ON_RESTART)
