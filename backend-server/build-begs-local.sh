#! /bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

export PATH="$PATH:$DIR/../../peregrine-eard/compiler/target/release"

SRC="$DIR/egs-data/egs"
DST="$DIR/egs-data/begs"

awk 'length { print "-c '${SRC}'/"$0 }' ./input-eard-files.txt | \
xargs eard-compiler -o $DST/render16.eardo
