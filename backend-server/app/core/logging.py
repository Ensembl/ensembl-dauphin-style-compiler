"""
See the NOTICE file distributed with this work for additional information
regarding copyright ownership.


Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at
http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
"""

import sys
import logging
import syslog
from types import FrameType
from typing import cast
from loguru import logger
from core import config
from logging.handlers import SysLogHandler
from logging import StreamHandler

class InterceptHandler(logging.Handler):
    def emit(self, record: logging.LogRecord) -> None:  # pragma: no cover
        try:
            level = logger.level(record.levelname).name
        except ValueError:
            level = str(record.levelno)

        frame, depth = logging.currentframe(), 2
        while frame.f_code.co_filename == logging.__file__:  # noqa: WPS609
            frame = cast(FrameType, frame.f_back)
            depth += 1

        logger.opt(depth=depth, exception=record.exc_info).log(
            level, record.getMessage(),
        )

def get_handler(facility=None):
    if facility == None:
        facility = syslog.LOG_USER
    log_host = config.LOG_HOST
    log_port = config.LOG_PORT
    if log_host == None:
        return StreamHandler(sys.stderr)
    else:
        return SysLogHandler(address=(log_host,log_port),facility=facility)

def setup_logging():
    LOGGING_LEVEL = logging.DEBUG if config.DEBUG else logging.WARN
    LOGGERS = ("uvicorn.asgi", "uvicorn.access",None)

    logging.getLogger().handlers = [InterceptHandler()]
    for logger_name in LOGGERS:
        logging_logger = logging.getLogger(logger_name)
        logging_logger.handlers = [get_handler()]
        logging_logger.setLevel(LOGGING_LEVEL)

special_loggers = {}
def get_logger(name,facility=None,level=None):
    global special_loggers

    LOGGING_LEVEL = logging.DEBUG if config.DEBUG else logging.WARN
    if name in special_loggers:
        return special_loggers[name]
    if level == None:
        level = LOGGING_LEVEL
    logger = logging.getLogger(name)
    logger.handlers = [get_handler(facility)]
    logger.setLevel(level)
    logger.propagate = False
    special_loggers[name] = logger
    return logger
