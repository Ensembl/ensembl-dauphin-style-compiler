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

from fastapi import FastAPI
from prometheus_fastapi_instrumentator import Instrumentator, metrics
from starlette.middleware.cors import CORSMiddleware

from api.resources.routes import router
from core.config import API_PREFIX, ALLOWED_HOSTS, VERSION, PROJECT_NAME, DEBUG

LATENCY_BUCKETS = (
    0.01,
    0.025,
    0.05,
    0.1,
    0.25,
    0.5,
    1,
    1.25,
    1.5,
    1.75,
    2,
    2.5,
    5,
    10,
    30,
)


def get_application() -> FastAPI:
    application = FastAPI(title=PROJECT_NAME, debug=DEBUG, version=VERSION)

    application.add_middleware(
        CORSMiddleware,
        allow_origins=ALLOWED_HOSTS or ["*"],
        allow_credentials=True,
        allow_methods=["GET","POST","OPTIONS"],
        allow_headers=["*"],
    )

    application.include_router(router, prefix=API_PREFIX)

    Instrumentator(excluded_handlers=["/metrics"]).add(
        metrics.default(latency_lowr_buckets=LATENCY_BUCKETS)
    ).instrument(application).expose(application, include_in_schema=False)

    return application


app = get_application()
