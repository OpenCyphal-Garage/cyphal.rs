#!/bin/bash

cd "uavcan"
cargo clean
cargo ${ACTION} ${FLAGS}
