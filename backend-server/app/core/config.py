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

import logging
from os import environ
import sys
from typing import List

from loguru import logger
from starlette.config import Config
from starlette.datastructures import CommaSeparatedStrings

from core.logging import InterceptHandler, setup_logging
from inspect import getsourcefile
from os.path import abspath
import os.path

current_directory = abspath(getsourcefile(lambda:0))
base_directory = os.path.join(os.path.dirname(current_directory),"..","..")

VERSION = "0.0.0"
API_PREFIX = "/api"

config = Config(".env")
DEBUG: bool = config("DEBUG", cast=bool, default=False)
LOG_HOST = config("LOG_HOST",default=None)
LOG_PORT = int(config("LOG_PORT",default=514))
PROJECT_NAME: str = config("PROJECT_NAME", default="Peregrine Data Server")
ALLOWED_HOSTS: List[str] = config(
    "ALLOWED_HOSTS", cast=CommaSeparatedStrings, default="*",
)

config_directory = config("CONFIG_DIRECTORY", default=os.path.join(base_directory,"config"))
egs_directory = config("EGS_DIRECTORY", default=os.path.join(base_directory,"egs-data","egs"))

EGS_FILES: str = config("EGS_FILES", default=egs_directory)
EGS_GLOBS: List[str] = ["*.egs"]
BEGS_CONFIG: str = config("BEGS_CONFIG", default=os.path.join(egs_directory,"begs_config.toml"))
BEGS_FILES: str = config("BEGS_FILES", default=os.path.join(base_directory,"egs-data","begs"))

METRIC_FILE = config("METRIC_FILE",default=os.path.join(base_directory,"metric.log"))

# logging configuration

SOURCES_TOML: str = config("SOURCES_TOML", default=os.path.join(config_directory,"sources-s3.toml"))

MEMCACHED = config("MEMCACHED", default="127.0.0.1:11211")

LO_PORT = config("LO_PORT",default=False)

setup_logging()
