#! /bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

export PATH="$PATH:$DIR/../dauphin/target/release"

SRC="$DIR/egs-data/egs"
DST="$DIR/egs-data/begs"

#dauphin -c $SRC/variant.egs -c $SRC/contig.egs -c $SRC/zoomed-seq.egs -c $SRC/gene.egs -c $SRC/transcript.egs -c $SRC/gene-overview.egs -c $SRC/gc.egs -c $SRC/zoomed-transcript-6.egs -c $SRC/ruler.egs -c $SRC/framing.egs -o $DST/render8.begs -L peregrine -O2 
dauphin -c $SRC/contig8.egs -c $SRC/ruler8.egs -c $SRC/framing8.egs -o $DST/render8.begs -L peregrine -O2 
dauphin -c $SRC/startup8.egs -c $SRC/lookup.egs -c $SRC/jump.egs -o $DST/stick8.begs -L peregrine -O2
dauphin -c $SRC/boot.egs -o $DST/boot8.begs -L peregrine -O2
