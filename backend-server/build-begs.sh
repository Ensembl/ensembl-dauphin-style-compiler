#! /bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

export PATH="$PATH:$DIR/../dauphin/target/release"

SRC="$DIR/egs-data/egs"
DST="$DIR/egs-data/begs"

dauphin -c $SRC/zoomed-seq9.egs -c $SRC/gene-overview9.egs -c $SRC/gene9.egs -c $SRC/transcript9.egs -c $SRC/zoomed-transcript9.egs -c $SRC/gc9.egs -c $SRC/variant9.egs -c $SRC/contig9.egs -c $SRC/ruler9.egs -c $SRC/framing9.egs -o $DST/render9.begs -L peregrine -O2 
dauphin -c $SRC/startup8.egs -c $SRC/lookup.egs -c $SRC/jump.egs -o $DST/stick9.begs -L peregrine -O2
dauphin -c $SRC/boot.egs -o $DST/boot9.begs -L peregrine -O2
