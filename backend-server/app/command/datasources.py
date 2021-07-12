from .begs import BegsFiles
from model.datamodel import DataModel

class DataAccessor(object):
    def __init__(self):
        self.reload()

    def reload(self):
        self.begs_files = BegsFiles()
        self.data_model = DataModel()
