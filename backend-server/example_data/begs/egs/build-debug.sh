#! /bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

export PATH="$PATH:../../../../dauphin/target/debug"

dauphin -c $DIR/gene.egs -c $DIR/gene-overview.egs -o $DIR/../render.begs -L peregrine -g
dauphin -c $DIR/startup.egs -c $DIR/lookup.egs -o $DIR/../stick.begs -L peregrine -g
dauphin -c $DIR/boot.egs -o $DIR/../boot.begs -L peregrine -g
