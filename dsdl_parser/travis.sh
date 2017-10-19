#!/bin/bash
set -x

cargo clean
cargo ${ACTION} ${FLAGS}
