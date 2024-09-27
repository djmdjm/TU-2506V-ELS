#!/bin/sh

# Build and write firmware to board using a ST-Link V2 or similar

cargo build --release
cargo flash --chip STM32F411CEUx
