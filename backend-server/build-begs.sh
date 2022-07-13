#! /bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

export PATH="$PATH:$DIR/../dauphin/target/release"

SRC="$DIR/egs-data/egs"
DST="$DIR/egs-data/begs"

dauphin -c $SRC/v11/zoomed-seq.egs -c $SRC/v11/gene-overview.egs -c $SRC/v11/gene.egs -c $SRC/v11/transcript.egs -c $SRC/v11/zoomed-transcript.egs -c $SRC/v11/gc.egs -c $SRC/v11/variant.egs -c $SRC/v11/contig.egs -c $SRC/v11/ruler.egs -c $SRC/v11/framing.egs -c $SRC/v11/focus-transcript.egs -c $SRC/v11/focus-zoomed-transcript.egs -o $DST/render11.begs -L peregrine -O2 
dauphin -c $SRC/v11/startup.egs -c $SRC/v11/lookup.egs -c $SRC/v11/jump.egs -o $DST/stick11.begs -L peregrine -O2
dauphin -c $SRC/v11/boot.egs -o $DST/boot11.begs -L peregrine -O2
