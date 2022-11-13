#! /bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

export PATH="$PATH:$DIR/../dauphin/target/release"

SRC="$DIR/egs-data/egs"
DST="$DIR/egs-data/begs"

dauphin -c $SRC/v15/zoomed-seq.egs -c $SRC/v15/gene-overview.egs -c $SRC/v15/gene.egs -c $SRC/v15/transcript.egs -c $SRC/v15/zoomed-transcript.egs -c $SRC/v15/gc.egs -c $SRC/v15/variant.egs -c $SRC/v15/contig.egs -c $SRC/v15/ruler.egs -c $SRC/v15/framing.egs -c $SRC/v15/focus-transcript.egs -c $SRC/v15/focus-zoomed-transcript.egs -o $DST/render15.begs -L peregrine -O2 
dauphin -c $SRC/v15/test.egs -c $SRC/v15/test-with-data.egs -o $DST/test15.begs -L peregrine -O2 