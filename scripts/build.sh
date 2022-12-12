#!/usr/bin/env bash

set -e
RFLAGS='-C link-arg=-s'
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
echo $SCRIPT_DIR

RELEASE_DIR=$SCRIPT_DIR/../target/wasm32-unknown-unknown/release
RES_DIR=$SCRIPT_DIR/res

echo "removing old res directory"
echo "rm -rf $RES_DIR"
rm -rf $RES_DIR


cd $SCRIPT_DIR/../butter-core
echo "start to build butter core"
RUSTFLAGS=$RFLAGS cargo build --target wasm32-unknown-unknown --release


cd $SCRIPT_DIR/../butter-core-factory
echo "start to build butter core factory"
RUSTFLAGS=$RFLAGS cargo build --target wasm32-unknown-unknown --release


mkdir $RES_DIR
cp $RELEASE_DIR/*.wasm $RES_DIR