import cbor2

from command.datasources import DataAccessor
from command.exceptionres import DataException
from command.response import Response
from model.chromosome import Chromosome
from model.version import Version
from util.influx import ResponseMetrics

class Handler:
    def process(self, data_accessor: DataAccessor, channel,  payload, metrics: ResponseMetrics, version: Version) -> Response:
        raise NotImplementedError("override process!")

    def remote_prefix(self, payload) -> list[str] | None:
        return None

class Panel(object):
    def __init__(self, data):
        (self.stick,self.scale,self.index) = data
        self.start = (1<<self.scale)*self.index
        self.end = (1<<self.scale)*(self.index+1)
    
    def get_chrom(self, data_accessor: DataAccessor) -> Chromosome:
        chrom = data_accessor.data_model.stick(self.stick)
        if chrom == None:
            raise DataException(f"Unknown chromosome: {self.stick}")
        return chrom

    def dumps(self):
        return cbor2.dumps([self.stick, self.scale, self.index])

class DataHandler:
    def get_scope(self, scope, key:str) -> str | None:
        return scope.get(key, [None])[0]

    def get_datafile(self, scope) -> str:
        filename = self.get_scope(scope, "datafile")
        if not filename:
            raise DataException("No datafile specified")
        return filename

    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope: dict, accept: str) -> dict:
        raise NotImplementedError("override process_data!")
