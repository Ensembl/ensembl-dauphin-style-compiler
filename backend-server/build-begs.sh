#! /bin/bash

SRC="./egs-data/egs/v16"
DST="./egs-data/begs"

eard-compiler \
    -c $SRC/other/framing.eard -c $SRC/other/gc.eard -c $SRC/other/contig.eard \
    -c $SRC/other/zoomed-seq.eard \
    -c $SRC/variant/variant-summary.eard -c $SRC/variant/variant-details.eard \
    -c $SRC/other/ruler.eard -c $SRC/gene/gene-overview.eard -c $SRC/gene/gene.eard \
    -c $SRC/gene/focus-gene.eard -c $SRC/gene/focus-gene-dots.eard \
    -c $SRC/gene/transcript.eard  -c $SRC/other/focus-region.eard \
    -c $SRC/gene/focus-transcript.eard -c $SRC/gene/zoomed-transcript.eard \
    -c $SRC/gene/focus-zoomed-transcript.eard -c $SRC/other/focus-region.eard \
    -c $SRC/variant/focus-variant.eard -c $SRC/variant/focus-variant-summary.eard \
    -c $SRC/variant/focus-variant-dots.eard -c $SRC/regulation/regulation.eard \
    -c $SRC/compara/compara-summary.eard -c $SRC/compara/compara-details.eard \
    -c $SRC/repeat/repeat-summary.eard -c $SRC/repeat/repeat-details.eard \
    -c $SRC/simple-features/tssp.eard -c $SRC/simple-features/cpg.eard -c $SRC/simple-features/trna.eard \
    -c $SRC/gene/sv-gene.eard \
    -o $DST/render16.eardo
