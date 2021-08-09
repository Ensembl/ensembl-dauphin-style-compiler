from .begs import BegsFiles
from model.datamodel import DataModel
from model.datalocator import DataSourceResolver

class DataAccessor(object):
    def __init__(self):
        self.reload()

    def reload(self):
        self.resolver : DataSourceResolver = DataSourceResolver()
        self.begs_files = BegsFiles()
        self.data_model = DataModel()
