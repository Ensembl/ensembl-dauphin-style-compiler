#! /bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

export PATH="$PATH:$DIR/../dauphin/target/release"

SRC="$DIR/egs-data/egs"
DST="$DIR/egs-data/begs"

dauphin -c $SRC/v14/zoomed-seq.egs -c $SRC/v14/gene-overview.egs -c $SRC/v14/gene.egs -c $SRC/v14/transcript.egs -c $SRC/v14/zoomed-transcript.egs -c $SRC/v14/gc.egs -c $SRC/v14/variant.egs -c $SRC/v14/contig.egs -c $SRC/v14/ruler.egs -c $SRC/v14/framing.egs -c $SRC/v14/focus-transcript.egs -c $SRC/v14/focus-zoomed-transcript.egs -o $DST/render14.begs -L peregrine -O2 
dauphin -c $SRC/v14/startup.egs -c $SRC/v14/lookup.egs -c $SRC/v14/jump.egs -o $DST/stick14.begs -L peregrine -O2
dauphin -c $SRC/v14/boot.egs -o $DST/boot14.begs -L peregrine -O2
