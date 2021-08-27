from typing import Any, List, Optional
from command.datasources import DataAccessor
import cbor2
from .response import Response

class Handler:
    def process(self, data_accessor: DataAccessor, channel: Any,  payload: Any) -> Response:
        raise NotImplementedError("override process!")

    def remote_prefix(self, payload: Any) -> Optional[List[str]]:
        return None

class Panel(object):
    def __init__(self, data):
        (self.stick,self.scale,self.index) = data
        self.start = (1<<self.scale)*self.index
        self.end = (1<<self.scale)*(self.index+1)

    def dumps(self):
        return cbor2.dumps([self.stick,self.scale,self.index])

class DataHandler:
    def process_data(self, data_accessor: DataAccessor, panel: Panel) -> Response:
        raise NotImplementedError("override process_data!")
