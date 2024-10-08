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
    -c $SRC/v16/variant/focus-variant-dots.eard -c $SRC/v16/regulation/regulation.eard \
    -c $SRC/v16/compara/compara-summary.eard -c $SRC/v16/compara/compara-details.eard \
    -c $SRC/v16/repeat/repeat-summary.eard -c $SRC/v16/repeat/repeat-details.eard \
    -c $SRC/v16/simple-features/tssp.eard -c $SRC/v16/simple-features/cpg.eard -c $SRC/v16/simple-features/trna.eard \
    -o $DST/render16.eardo
