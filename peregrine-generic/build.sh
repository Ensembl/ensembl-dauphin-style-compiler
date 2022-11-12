#! /bin/bash

set -e

SCRIPT=$(readlink -f "$0")
SCRIPTPATH=$(dirname "$SCRIPT")
cd $SCRIPTPATH

# configure
$SCRIPTPATH/../build-tools/menu.py --quick --use-prev=.config.prev build-menu.json .cfg || exit
source .cfg

FLAGS=""

if [ "x$CFG_DEBUG_WEBGL" = "xyes" ] ; then
  FLAGS="$FLAGS --cfg debug_webgl"
fi
if [ "x$CFG_DEBUG_TIMEHOGS" = "xyes" ] ; then
  FLAGS="$FLAGS --cfg debug_timehogs"
fi
if [ "x$CFG_DEBUG_SAMPLER" = "xyes" ] ; then
  FLAGS="$FLAGS --cfg debug_sampler"
fi
if [ "x$CFG_DEBUG_TRAINS" = "xyes" ] ; then
  FLAGS="$FLAGS --cfg debug_trains"
fi
if [ "x$CFG_DEBUG_BIG_REQUESTS" = "xyes" ] ; then
  FLAGS="$FLAGS --cfg debug_big_requests"
fi
if [ "x$CFG_DEBUG_DATA_REQUESTS" = "xyes" ] ; then
  FLAGS="$FLAGS --cfg debug_data_requests"
fi
if [ "x$CFG_DISABLE_ANTICIPATE" = "xyes" ] ; then
  FLAGS="$FLAGS --cfg disable_anticipate"
fi
if [ "x$CFG_NO_FLANK" = "xyes" ] ; then
  FLAGS="$FLAGS --cfg no_flank"
fi
if [ "x$CFG_DEBUG_CLEANUP" = "xyes" ] ; then
  FLAGS="$FLAGS --cfg debug_unregister"
fi
if [ "x$FORCE_DPR" != "x" ] && [ "x$FORCE_DPR_YN" = "xyes" ] ; then
  export FORCE_DPR="$FORCE_DPR"
fi

case "$CFG_CONSOLE" in
  noisy)
    FLAGS="$FLAGS --cfg console_noisy"
    ;;
  quiet)
    FLAGS="$FLAGS --cfg console_quiet"
    ;;
  *)
    ;;
esac

echo RUSTFLAGS="$FLAGS" wasm-pack build --target web --$CFG_RUST_MODE
RUSTFLAGS="$FLAGS" wasm-pack build --target web --$CFG_RUST_MODE

if [ ! "x$CFG_PORT" = "x0" ] ; then
  echo "killing old server"
  lsof -t -i:$CFG_PORT | xargs -r kill
  sleep 2
fi
python3 server.py $CFG_PORT
