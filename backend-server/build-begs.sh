#! /bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

export PATH="$PATH:$DIR/../dauphin/target/release"

SRC="$DIR/egs-data/egs"
DST="$DIR/egs-data/begs"

dauphin -c $SRC/v10/zoomed-seq.egs -c $SRC/v10/gene-overview.egs -c $SRC/v10/gene.egs -c $SRC/v10/transcript.egs -c $SRC/v10/zoomed-transcript.egs -c $SRC/v10/gc.egs -c $SRC/v10/variant.egs -c $SRC/v10/contig.egs -c $SRC/v10/ruler.egs -c $SRC/v10/framing.egs -o $DST/render10.begs -L peregrine -O2 
dauphin -c $SRC/v10/startup.egs -c $SRC/v10/lookup.egs -c $SRC/v10/jump.egs -o $DST/stick10.begs -L peregrine -O2
dauphin -c $SRC/v10/boot.egs -o $DST/boot10.begs -L peregrine -O2
