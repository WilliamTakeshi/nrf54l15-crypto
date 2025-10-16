#!/usr/bin/env bash
set -e

APP_CORE_DIR=app-core
FLPR_CORE_DIR=flpr-core


APP_CORE_BIN=configure-vpr-core # blinky, blinky-pac, temp, etc...

pushd $APP_CORE_DIR
cargo build --release
popd

pushd $FLPR_CORE_DIR
cargo build --release
popd

# To use rust-objcopy, the cargo-binutils tools need to be installed. (rustup component add llvm-tools)
rust-objcopy -O ihex target/thumbv8m.main-none-eabihf/release/$APP_CORE_BIN app-core.hex
probe-rs download app-core.hex --chip nRF54L15 --binary-format hex

rust-objcopy -O ihex target/riscv32emc-unknown-none-elf/release/blinky flpr-core.hex
probe-rs download flpr-core.hex --chip nRF54L15 --binary-format hex

probe-rs reset --chip nRF54L15
