#!/bin/bash
set -x

cd "uavcan"
cargo clean
cargo ${ACTION} ${FLAGS}
