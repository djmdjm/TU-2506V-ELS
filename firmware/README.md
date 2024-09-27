# Firmware for Electronic Leadscrew

This is the firmware for the electronic leadscrew controller board.
It's core task is receiving pulses from the encoder connected to the
lathe spindle and emitting pulses to drive the servo motor attached to
the lathe's leadscrew.

The user interface is a 16x2 VFD character display, two rotary encoders
with integrated pushbuttons and a dedicated pushbutton.

The firmware is written in Rust targetting the STM32F411 MCU. If you've
installed Rust using rustup, then you can get the toolchain and required
tools using:

```shell
rustup target add thumbv7em-none-eabihf
cargo install cargo-binutils
cargo install probe-rs-tools
```

For a good background on Rust embedded development, how to set up openocd,
install prerequisite system packages, etc. the
[Embedded Rust Book](https://docs.rust-embedded.org/book/) is a great start.

To build the firmware, run `cargo build` from this directory. If you have
the board attached using a ST-Link dongle and have `openocd` running,
then `cargo run` will attempt to download the firmware to the board and
run it under gdb. The `flash.sh` script will burn a release version of the
firmware to the board.

