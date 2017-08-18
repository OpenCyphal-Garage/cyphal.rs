#!/bin/bash

cd "uavcan"
cargo clean
cargo test ${FLAGS}
