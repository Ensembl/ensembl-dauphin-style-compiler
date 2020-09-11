#! /bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

dauphin -c $DIR/gene.egs -o $DIR/../render.begs -L peregrine -O2
dauphin -c $DIR/startup.egs -c $DIR/lookup.egs -o $DIR/../stick.begs -L peregrine -O2
dauphin -c $DIR/boot.egs -o $DIR/../boot.begs -L peregrine -O2
