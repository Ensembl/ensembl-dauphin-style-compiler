from .coremodel import Handler
from .datasources import DataAccessor
from typing import Any, List, Optional
from core.config import METRIC_FILE
from .response import Response
import datetime
from core.logging import get_logger
import logging, socket
from util.influx import send_to_telegraf, ResponseMetrics
from model.version import Version

FACILITIES = {
    "Error": 17,
    "Metric": 18
}

class GeneralMetricHandler:
    def process_metric(self,message_type,payload):
        lines = ""
        for (name,tag_keys,tag_values,value_keys,data) in payload['general'].items():
            for (tag_keys_data,tag_values_data,value_keys_data,value_values) in data:
                tags = [ (tag_keys[k],tag_values[v]) for (k,v) in zip(tag_keys,tag_values) ]
                values = [ (value_keys[k],v) for (k,v) in zip(value_keys,value_values) ]
                tags_str = ",".join(["{0}={1}".format(k,v) for (k,v) in tags])
                values_str = ",".join(["{0}={1}".format(k,v) for (k,v) in values])
                lines += "{0},{1} {2}\n".format(name,tags_str,values_str)
        send_to_telegraf(lines)

class DatastreamMetricHandler:
    def unmangle(self,payload):
        out = []
        datastream = payload['datastream']
        for datapoint in datastream['datapoints']:
            out.append({
                "name": datastream['names'][datapoint[0]],
                "key": datastream['keys'][datapoint[1]],
                "scale": datapoint[2],
                "priority": "batch" if datapoint[3] else "realtime",
                "num-events": datapoint[4],
                "total_size": datapoint[5]
            })
        return out

    def to_influx(self,message_type,payload):
        lines = ""
        for metric in payload:
            values = {
                "count": metric["num-events"],
                "bytes": metric["total_size"]
            }
            if metric["num-events"] > 0:
                values['bpc'] = metric["total_size"] / metric["num-events"]
            values_str = ",".join(["{0}={1}".format(k,v) for (k,v) in values.items()])
            lines += "gb-requests,name={0},key={1},scale={2},priority={3} {4}\n".format(
                metric["name"],
                metric["key"],
                metric["scale"],
                metric["priority"],
                values_str
            )
        return lines

    def process_metric(self,message_type,payload):
        payload = self.unmangle(payload)
        lines = self.to_influx(message_type,payload)
        send_to_telegraf(lines)

class ProgramRunMetricHandler:
    def unmangle(self,payload):
        out = []
        programrun = payload['programrun']
        for datapoint in programrun['datapoints']:
            out.append({
                "name": programrun['names'][datapoint[0]],
                "scale": datapoint[1],
                "warm": datapoint[2],
                "net_ms": datapoint[3],
                "time_ms": datapoint[4]
            })
        return out


    def to_influx(self,message_type,payload):
        lines = ""
        for metric in payload:
            lines += "prog-time,name={0},scale={1},warm={2} net_ms={3},time_ms={4}\n".format(
                metric["name"],
                metric["scale"],
                metric["warm"],
                metric["net_ms"],
                metric["time_ms"]-metric["net_ms"],
            )
        return lines

    # XXX disablable
    # XXX expiry
    # XXX config
    # XXX try/except
    def process_metric(self,message_type,payload):
        payload = self.unmangle(payload)
        lines = self.to_influx(message_type,payload)
        send_to_telegraf(lines)

class LoggingMetricHandler:
    def process_metric(self,message_type,payload):
        logger_name = "metric" + message_type.lower()
        logger = get_logger(logger_name,facility=FACILITIES.get(message_type,None),level=logging.INFO)
        logger.info(payload)

METRIC_HANDLERS = {
    "Client": [DatastreamMetricHandler(),ProgramRunMetricHandler(),GeneralMetricHandler()],
    "": [LoggingMetricHandler()]
}

class MetricHandler(Handler):
    def process(self, data_accessor: DataAccessor, channel: Any, payload: Any, metrics: ResponseMetrics, version: Version) -> Response:
        message_type = payload["type"]
        handlers = METRIC_HANDLERS.get(message_type,METRIC_HANDLERS[""])
        for handler in handlers:
            handler.process_metric(message_type,payload)
        r = Response(2,[])
        return r

    def remote_prefix(self, payload: Any) -> Optional[List[str]]:
        return ["metric"]
