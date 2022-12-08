#! /bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

export PATH="$PATH:$DIR/../dauphin/target/release"

SRC="$DIR/egs-data/egs"
DST="$DIR/egs-data/begs"

dauphin -c $SRC/v16/focus-gene-dots.egs -c $SRC/v16/focus-region.egs -c $SRC/v16/focus-gene.egs -c $SRC/v16/zoomed-seq.egs -c $SRC/v16/gene-overview.egs -c $SRC/v16/gene.egs -c $SRC/v16/transcript.egs -c $SRC/v16/zoomed-transcript.egs -c $SRC/v16/gc.egs -c $SRC/v16/variant.egs -c $SRC/v16/contig.egs -c $SRC/v16/ruler.egs -c $SRC/v16/framing.egs -c $SRC/v16/focus-transcript.egs -c $SRC/v16/focus-zoomed-transcript.egs -o $DST/render16.begs -L peregrine -O2 
dauphin -c $SRC/v16/test.egs -c $SRC/v16/test-with-data.egs -o $DST/test16.begs -L peregrine -O2 