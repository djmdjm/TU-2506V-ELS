# Controller schematics and PCB designs

This directory contains the schematics and PCB CAD files for the controller
boards. To fit in the available space in the lathe casting they are designed
as a stack.

The `mcu/` subdirectory contains the designs for the MCU carrier board, which
basically exists to route signals from the STM32F411 Black Pill board to the
adjacent ones via headers. The character VFD display also connects to this
board via extended headers.

The `pwr_io/` subdirectory has the design for the isolated power and I/O board.
It has a couple of modular DC-DC converters that provide an isolated supply
to the MCU and display, and a separate isolated supply for the spindle encoder
and servo control lines.

The `ui/` subdirectory has the design for the carrier board for the rotary
encoders used on the front panel (you'll need two of these). These connect
via cables terminated in IDC connectors on the MCU board.
