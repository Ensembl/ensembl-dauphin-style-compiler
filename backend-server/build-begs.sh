#! /bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

export PATH="$PATH:$DIR/../dauphin/target/release"

SRC="$DIR/egs-data/egs"
DST="$DIR/egs-data/begs"

dauphin -c $SRC/v12/zoomed-seq.egs -c $SRC/v12/gene-overview.egs -c $SRC/v12/gene.egs -c $SRC/v12/transcript.egs -c $SRC/v12/zoomed-transcript.egs -c $SRC/v12/gc.egs -c $SRC/v12/variant.egs -c $SRC/v12/contig.egs -c $SRC/v12/ruler.egs -c $SRC/v12/framing.egs -c $SRC/v12/focus-transcript.egs -c $SRC/v12/focus-zoomed-transcript.egs -o $DST/render12.begs -L peregrine -O2 
dauphin -c $SRC/v12/startup.egs -c $SRC/v12/lookup.egs -c $SRC/v12/jump.egs -o $DST/stick12.begs -L peregrine -O2
dauphin -c $SRC/v12/boot.egs -o $DST/boot12.begs -L peregrine -O2
