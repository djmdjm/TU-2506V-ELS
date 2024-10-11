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

## Modes

The firmware supports a number of operating modes and a debug mode.
Switching between modes is done using one of the control rotary encoders,
with the other rotary encoder being used to set parameters within that mode
such as feed rate or thread pitch. Mode changes, and some parameter changes
will be ignored unless the spindle is stationery.

The controller starts in the `Servo off` mode, which unsurprisingly disables
the servo drive. The `Feed` mode is intended for general cutting applications,
and runs the leadscrew to advance at a configured rate (in mm) per spindle
revolution. Two threading modes are available: `Thread`, which performs metric
(mm/rev) threading and `Thread Im` which does imperial (TPI) threading.

The debug mode can be accessed by pressing the mode encoder's button for a
couple of seconds. This mode has a number of pages, accessible via the mode
wheel, that show various internal debugging parameters. Debug mode can be
exited by holding down the mode button again.

## Notes

The control loop for the ELS is quite simplistic - the mainloop runs
continuously and (depending on operating mode) generates servo control pulses
as quickly as it receives encoder pulses. It uses mostly-precomputed 64-bit
math to simplify (as 32.32 fixed point) to simplify calculations. Once of the
nice things about having a 100MHz MCU is not having to worry too much about
the cycle cost here.

There is one interrupt handler hooked up to one of the timers that runs a 1KHz
monotonic counter that is used to drive the 10Hz display update and the RPM
sampling/smoothing.

The most time-critical component of the firmware is the servo pulse generation.
At high RPM and high feed rates, the firmware needs to be able to quickly emit
lots of pulses. Conversely, the servo requires a maximum pulse frequency of
500KHz. I couldn't get this running fast enough in Rust, so the actual pulse
generation is written in assembly.

Motor, encoder, leadscrew and gear/pulley ratio constants are in `src/main.rs`.
You'll almost certainly need to edit these.

The calculations are unashamedly metric and inch leadscrews are not supported.

There's currently no detection of situations where the control loop can't keep
up (i.e. can't generate servo pulses fast enough for a given spindle RPM).
I'm fairly sure this situation isn't possible with the parameters I've used,
and I've certainly not encountered it in use (though admittedly I haven't
tried to run the leadscrew at 4mm/rev and 1500RPM).
