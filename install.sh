#! /bin/bash

# A convenience script for building and installing ducker
# Pulls head of master, builds & places in /usr/local/bin




SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

pushd $SCRIPT_DIR

git pull
cargo build -r
sudo cp target/release/ducker /usr/local/bin/ducker

popd

