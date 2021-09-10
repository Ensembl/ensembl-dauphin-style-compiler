from .coremodel import Handler
from .datasources import DataAccessor
from typing import Any
from core.config import METRIC_FILE
from .response import Response
import datetime
from core.logging import get_logger
import logging

FACILITIES = {
    "Error": 17,
    "Metric": 18
}

class MetricHandler(Handler):
    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any) -> Response:
        message_type = payload["type"]
        logger_name = "metric" + message_type.lower()
        logger = get_logger(logger_name,facility=FACILITIES.get(message_type,None),level=logging.INFO)
        logger.info(payload)
        r = Response(2,[])
        return r
