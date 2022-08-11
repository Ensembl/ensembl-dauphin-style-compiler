#! /bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

export PATH="$PATH:$DIR/../dauphin/target/release"

SRC="$DIR/egs-data/egs"
DST="$DIR/egs-data/begs"

dauphin -c $SRC/v13/zoomed-seq.egs -c $SRC/v13/gene-overview.egs -c $SRC/v13/gene.egs -c $SRC/v13/transcript.egs -c $SRC/v13/zoomed-transcript.egs -c $SRC/v13/gc.egs -c $SRC/v13/variant.egs -c $SRC/v13/contig.egs -c $SRC/v13/ruler.egs -c $SRC/v13/framing.egs -c $SRC/v13/focus-transcript.egs -c $SRC/v13/focus-zoomed-transcript.egs -o $DST/render13.begs -L peregrine -O2 
dauphin -c $SRC/v13/startup.egs -c $SRC/v13/lookup.egs -c $SRC/v13/jump.egs -o $DST/stick13.begs -L peregrine -O2
dauphin -c $SRC/v13/boot.egs -o $DST/boot13.begs -L peregrine -O2
