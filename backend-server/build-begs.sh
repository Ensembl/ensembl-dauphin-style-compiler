#! /bin/bash

if ! command -v eard-compiler &> /dev/null
then
    export PATH="$PATH:../../peregrine-eard/compiler/target/release"
fi

SRC="./egs-data/egs"
DST="./egs-data/begs"

eard-compiler \
    -c $SRC/v16/other/framing.eard -c $SRC/v16/other/gc.eard -c $SRC/v16/other/contig.eard \
    -c $SRC/v16/other/zoomed-seq.eard -c $SRC/v16/variant/variant-1000genomes.eard \
    -c $SRC/v16/variant/variant-dbsnp.eard -c $SRC/v16/variant/variant-eva.eard -c $SRC/v16/variant/variant-sgrp.eard \
    -c $SRC/v16/variant/variant-clinvar.eard -c $SRC/v16/variant/variant-gwas.eard \
    -c $SRC/v16/other/ruler.eard -c $SRC/v16/gene/gene-overview.eard -c $SRC/v16/gene/gene.eard \
    -c $SRC/v16/gene/focus-gene.eard -c $SRC/v16/gene/focus-gene-dots.eard \
    -c $SRC/v16/gene/transcript.eard  -c $SRC/v16/other/focus-region.eard \
    -c $SRC/v16/gene/focus-transcript.eard -c $SRC/v16/gene/zoomed-transcript.eard \
    -c $SRC/v16/gene/focus-zoomed-transcript.eard -c $SRC/v16/other/focus-region.eard \
    -c $SRC/v16/variant/focus-variant.eard -c $SRC/v16/variant/focus-variant-summary.eard \
    -c $SRC/v16/variant/focus-variant-dots.eard \
    -o $DST/render16.eardo
