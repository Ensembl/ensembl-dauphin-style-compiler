#! /usr/bin/env bash

# Always run one directory up from this script
# 1. where is this script?
SCRIPT=$(readlink -f "$0")
# Absolute path this script is in, thus /home/user/bin
SCRIPTPATH=$(dirname "$SCRIPT")
# 2. go one directory up
cd $SCRIPTPATH/..

DOCKER_BUILDKIT=1 docker build -f peregrine-ensembl/Dockerfile .
