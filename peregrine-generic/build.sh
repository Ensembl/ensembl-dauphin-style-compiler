#! /bin/bash

set -e

# Always run one directory up from this script
# 1. where is this script?
SCRIPT=$(readlink -f "$0")
# Absolute path this script is in, thus /home/user/bin
SCRIPTPATH=$(dirname "$SCRIPT")
# 2. go one directory up
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


