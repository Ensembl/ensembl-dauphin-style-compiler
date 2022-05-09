#! /bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

export PATH="$PATH:$DIR/../dauphin/target/release"

SRC="$DIR/egs-data/egs"
DST="$DIR/egs-data/begs"

dauphin -c $SRC/zoomed-seq8.egs -o $DST/render8.begs -L peregrine -O2 
