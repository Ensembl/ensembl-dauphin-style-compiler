#! /bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

export PATH="$PATH:$DIR/../dauphin/target/debug"

SRC="$DIR/egs-data/egs"
DST="$DIR/egs-data/begs"

dauphin -c $SRC/gene.egs -c $SRC/gene-overview.egs -c $SRC/gc.egs -o $DST/render.begs -L peregrine -O2 
dauphin -c $SRC/startup.egs -c $SRC/lookup.egs -o $DST/stick.begs -L peregrine -O2
dauphin -c $SRC/boot.egs -o $DST/boot.begs -L peregrine -O2

