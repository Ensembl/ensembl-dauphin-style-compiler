#! /bin/bash

cd app
uvicorn main:app --host 0.0.0.0 --port 3333 --workers 4

