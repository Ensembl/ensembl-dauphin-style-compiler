#! /bin/bash

# This new script should go through a friendly menu to configure things, but it's not been tested on a mac yet
# (though any changes required should be minor).To avoid breaking things, the old script is temporarily retained,
# named build-old.sh.

set -e

SCRIPT=$(readlink -f "$0")
SCRIPTPATH=$(dirname "$SCRIPT")
cd $SCRIPTPATH

# configure
$SCRIPTPATH/../build-tools/menu.py --use-prev=.config.prev build-menu.json .cfg
source .cfg

FLAGS=""

if [ "x$CFG_DEBUG_WEB_GL" = "xyes" ] ; then
  FLAGS="$FLAGS --cfg=debug_webgl"
fi

RUSTFLAGS="--cfg=console --cfg=force_show_incoming $FLAGS" wasm-pack build --target web --$CFG_RUST_MODE

if [ ! "x$CFG_PORT" = "x0" ] ; then
  echo "killing old server"
  lsof -t -i:$CFG_PORT | xargs -r kill
  sleep 2
fi
python3 server.py $CFG_PORT


