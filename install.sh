#! /bin/bash

# A convenience script for pulling, building and
# installing ducker;
# Relies on an installation of rust.
# Can also be used for updates

# Slightly round-about as could just use cargo install, but allows for incremental builds

mkdir -p ~/.ducker

cd ~/.ducker

if [ ! -d ./.git ];
then
    git clone --depth 1 https://github.com/robertpsoane/ducker ~/.ducker
fi


git checkout master
git reset --hard origin/master
git pull

cargo build -r
cargo install --locked --path .

clear

echo "🦆🦆🦆🦆🦆🦆🦆🦆🦆🦆🦆🦆"
echo "🦆 Ducker Installed! 🦆"
echo "🦆  Happy Quacking!  🦆"
echo "🦆🦆🦆🦆🦆🦆🦆🦆🦆🦆🦆🦆"
