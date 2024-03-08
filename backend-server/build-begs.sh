#! /bin/bash

SRC="./egs-data/egs"
DST="./egs-data/begs"

eard-compiler \
    -c $SRC/v16/other/framing.eard -c $SRC/v16/other/gc.eard -c $SRC/v16/other/contig.eard \
    -c $SRC/v16/other/zoomed-seq.eard \
    -c $SRC/v16/variant/variant-summary.eard -c $SRC/v16/variant/variant-details.eard \
    -c $SRC/v16/other/ruler.eard -c $SRC/v16/gene/gene-overview.eard -c $SRC/v16/gene/gene.eard \
    -c $SRC/v16/gene/focus-gene.eard -c $SRC/v16/gene/focus-gene-dots.eard \
    -c $SRC/v16/gene/transcript.eard  -c $SRC/v16/other/focus-region.eard \
    -c $SRC/v16/gene/focus-transcript.eard -c $SRC/v16/gene/zoomed-transcript.eard \
    -c $SRC/v16/gene/focus-zoomed-transcript.eard -c $SRC/v16/other/focus-region.eard \
    -c $SRC/v16/variant/focus-variant.eard -c $SRC/v16/variant/focus-variant-summary.eard \
    -c $SRC/v16/other/compara-elements.eard -c $SRC/v16/other/compara-scores.eard \
    -c $SRC/v16/variant/focus-variant-dots.eard \
    -c $SRC/v16/regulation/regulation.eard \
    -c $SRC/v16/other/tssp.eard \
    -o $DST/render16.eardo
