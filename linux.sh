#!/bin/bash
cargo build --release
tar -czf artifacts/Linux-BasePC2.tar.gz -C target/release evm
