#! /bin/bash

# This new script should go through a friendly menu to configure things, but it's not been tested on a mac yet
# (though any changes required should be minor).To avoid breaking things, the old script is temporarily retained,
# named build-old.sh.

set -e

SCRIPT=$(readlink -f "$0")
SCRIPTPATH=$(dirname "$SCRIPT")
cd $SCRIPTPATH

# configure
$SCRIPTPATH/../build-tools/menu.py --use-prev=.config.prev build-menu.json .cfg || exit
source .cfg

FLAGS=""

if [ "x$CFG_DEBUG_WEB_GL" = "xyes" ] ; then
  FLAGS="$FLAGS --cfg=debug_webgl"
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

