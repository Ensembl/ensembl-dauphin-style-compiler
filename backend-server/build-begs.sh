#! /bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

SRC="./egs-data/egs"
DST="./egs-data/begs"

#dauphin -c $SRC/v16/test.egs -c $SRC/v16/test-with-data.egs -o $DST/test16.begs -L peregrine -O2 
eard-compiler \
    -c $SRC/v16/other/framing.eard -c $SRC/v16/other/gc.eard -c $SRC/v16/other/contig.eard \
    -c $SRC/v16/other/zoomed-seq.eard -c $SRC/v16/variant/variant-summary.eard \
    -c $SRC/v16/other/ruler.eard -c $SRC/v16/gene/gene-overview.eard -c $SRC/v16/gene/gene.eard \
    -c $SRC/v16/gene/focus-gene.eard -c $SRC/v16/gene/focus-gene-dots.eard \
    -c $SRC/v16/gene/transcript.eard  -c $SRC/v16/other/focus-region.eard \
    -c $SRC/v16/gene/focus-transcript.eard -c $SRC/v16/gene/zoomed-transcript.eard \
    -c $SRC/v16/gene/focus-zoomed-transcript.eard -c $SRC/v16/other/focus-region.eard \
    -c $SRC/v16/variant/focus-variant.eard -c $SRC/v16/variant/focus-variant-summary.eard \
    -c $SRC/v16/variant/focus-variant-dots.eard \
    -o $DST/render16.eardo
