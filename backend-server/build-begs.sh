#! /bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

export PATH="$PATH:$DIR/../dauphin/target/release"
export PATH="$PATH:$DIR/../../eard/compiler/target/release"

SRC="$DIR/egs-data/egs"
DST="$DIR/egs-data/begs"

dauphin -c $SRC/v16/focus-gene-dots.egs -c $SRC/v16/focus-region.egs -c $SRC/v16/focus-gene.egs -c $SRC/v16/gene.egs -c $SRC/v16/transcript.egs -c $SRC/v16/zoomed-transcript.egs -c $SRC/v16/focus-transcript.egs -c $SRC/v16/focus-zoomed-transcript.egs -o $DST/render16.begs -L peregrine -O2 
dauphin -c $SRC/v16/test.egs -c $SRC/v16/test-with-data.egs -o $DST/test16.begs -L peregrine -O2 
eard-compiler -c $SRC/v16/framing.eard -c $SRC/v16/gc.eard -c $SRC/v16/contig.eard -c $SRC/v16/zoomed-seq.eard -c $SRC/v16/variant.eard -c $SRC/v16/ruler.eard -c $SRC/v16/gene-overview.eard -o $DST/render16.eardo
