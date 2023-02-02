import logging
from typing import Any
from model.version import Version
from command.begs import UnknownVersionException
from command.response import Response
from command.coremodel import Handler
from util.influx import ResponseMetrics
from command.datasources import DataAccessor
from core.config import SMALL_VALUE_TOML
import toml

class SmallValueHandler(Handler):
    def __init__(self):
        with open(SMALL_VALUE_TOML) as f:
            self._values = toml.loads(f.read())

    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any, metrics: ResponseMetrics, version: Version) -> Response:
        try:
            (namespace,column) = payload
            values = self._values.get(namespace,{}).get(column,{})
            r = Response(9,[values])
            r.add_values(namespace,column,values)
            return r
        except UnknownVersionException as e:
            logging.warn(e)
            return Response(1,e)
