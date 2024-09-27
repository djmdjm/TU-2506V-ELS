#!/bin/sh

set -xe
cargo build
cargo objcopy --release -- -O binary prog.bin
dfu-util -D prog.bin -d 0483:df11 -a 0 -s 0x08000000:leave
