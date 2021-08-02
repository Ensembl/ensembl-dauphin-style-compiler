#! /bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

export PATH="$PATH:$DIR/../dauphin/target/debug"

SRC="$DIR/egs-data/egs"
DST="$DIR/egs-data/begs"

dauphin -c $SRC/variant.egs -c $SRC/contig.egs -c $SRC/zoomed-seq.egs -c $SRC/gene.egs -c $SRC/transcript.egs -c $SRC/gene-overview.egs -c $SRC/gc.egs -c $SRC/zoomed-transcript.egs -o $DST/render.begs -L peregrine -O2 
dauphin -c $SRC/startup.egs -c $SRC/lookup.egs -c $SRC/jump.egs -o $DST/stick.begs -L peregrine -O2
dauphin -c $SRC/boot.egs -o $DST/boot.begs -L peregrine -O2

