#! /usr/bin/env bash

# Always run one directory up from this script
# 1. where is this script?
SCRIPT=$(readlink -f "$0")
# Absolute path this script is in, thus /home/user/bin
SCRIPTPATH=$(dirname "$SCRIPT")
# 2. go one directory up
cd $SCRIPTPATH/..

# check

docker images wasmpack --format='{{.ID}}' | grep -q ''
if [ $? -ne 0 ] ; then echo "no wasmpack image, please build it with build-wasmpack.sh" ; exit 1 ; fi

# setup
mkdir -p ./egb-tmp
tar -c -z -f egb.tar.gz --files-from /dev/null

# configure
$SCRIPTPATH/../build-tools/menu.py --quick --use-prev=.buildkit-config.prev $SCRIPTPATH/buildkit-menu.json .cfg || exit
source .cfg

if [ "x$CFG_EGB" = "xlocal" ] ; then
      if [[ ! -e ../ensembl-genome-browser ]] ; then echo "cannot find egb repo" ; exit 1 ; fi
      rm egb.tar.gz
      tar -c -z -v -f egb.tar.gz ../ensembl-genome-browser
fi

echo "rust build: $CFG_RUST_MODE   e-g-b: $CFG_EGB   clear cache: $CFG_CLEAR   backend: $CFG_BE   progress: ${CFG_PROGRESS:-pretty}"

case "$CFG_BE" in
  local)
    CFG_BE_URL="http://localhost:3333/api/data"
    ;;
  proxy)
    CFG_BE_URL="http://localhost:$CFG_PORT/api/browser/data"
    ;;
  staging)
    CFG_BE_URL="http://staging-2020.ensembl.org/api/browser/data"
    ;;
  aws)
    CFG_BE_URL="http://52.56.215.72:3333/api/data"
    ;;
esac

case "$CFG_PROGRESS" in
  plain)
    CFG_FLAGS="--progress=plain"
    ;;
  *)
    CFG_FLAGS=""
    ;;
esac

case "$CFG_EC_DAILY" in
  yes)
    CACHE_BUST="'$(date +%Y%m%d)'"
    ;;
  *)
    CACHE_BUST="'$(date +%Y%m%d%H%M%S)'"
    ;;
esac

CFG_RUSTFLAGS=""
case "$CFG_CONSOLE" in
  noisy)
    CFG_RUSTFLAGS="$CFG_RUSTFLAGS --cfg console_noisy"
    ;;
  quiet)
    CFG_RUSTFLAGS="$CFG_RUSTFLAGS --cfg console_quiet"
    ;;
  *)
    ;;
esac

# clear cache
if [ "x$CFG_CLEAR" == "xyes" ] ; then
  docker builder prune --filter type=exec.cachemount -f
fi

# build
DOCKER_BUILDKIT=1 docker build \
    --build-arg CFG_RUST_MODE=--$CFG_RUST_MODE --build-arg CFG_EGB=$CFG_EGB \
    --build-arg CACHE_DATE=$CACHE_BUST --build-arg FLAGS="$CFG_RUSTFLAGS" \
    -f peregrine-ensembl/Dockerfile-buildkit --iidfile /tmp/build.id $CFG_FLAGS .

# tidy
rm -rf egb.tar.gz

# kill old one
ID=$(docker ps -f publish=$CFG_PORT --format '{{.ID}}')
if [ ! -z "$ID" ]; then docker kill $ID ; fi

# start new one
if [ ! -e /tmp/build.id ] ; then echo "build failed" ; exit 1 ; fi
docker run -p $CFG_PORT:8080 -e GENOME_BROWSER_BACKEND_BASE_URL=$CFG_BE_URL $(cat /tmp/build.id) &
wait

