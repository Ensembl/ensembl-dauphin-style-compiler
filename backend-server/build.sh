#! /bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd $SCRIPT_DIR

cd app
export SOURCES_TOML=sources-s3.toml
uvicorn main:app --host 0.0.0.0 --port 3333 --workers 16
