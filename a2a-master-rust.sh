#!/bin/bash
source $HOME/a2a-multicloud/set_env.sh

cd rust-master

echo "Starting Rust A2A Master Agent on port 8100..."
cargo run
