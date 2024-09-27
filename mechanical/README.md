# CAD files for mechanical parts

This directory contains CAD files for some of the mechanical parts and
assemblies. They were all designed in FreeCAD 0.21. Some components
require plugins, e.g. the gears workbench and the fasteners workbench.

If you plan on using the CAM jobs included in some of the components,
you will definitiely need to change the machine model and tooling.

## Motor mounting plate

The `lathe_leadscrew_motor_plate.FCStd` file is the CAD for the servo
motor mounting plate. It's designed for a NEMA 23 servo or stepper.
It's a very basic part that is probably best laser-cut.

I did need to modify this plate a little to fit the space, but cutting
off a couple of the corners. This is not reflected in the CAD file.

This plate does double duty as a template to mark the locations of the
holes that need to be drilled and tapped into the lathe casting.

## Encoder assembly

The `lathe_leadscrew_encoder_plate.FCStd` is the assembly that couples
the encoder to the spindle. There's a few bespoke parts in here: a main
plate that holds the shaft bearing, a separate plate that mounts the
encoder, some standoffs to separate the two and a spindle that is driven
by a gear that couples to the spindle. Note the flexible coupler here
comes with the encoder.

I built the main plate as a weldment from laser-cut sheet metal and a
turned part that I then re-bored for concentricity on the CNC mill.

At first I used one of the original change gears on this assembly but,
like most change gears, it was annoyingly noisy. I ended up casting the
gear in polyurethane, which helped insofar as PU is better acoustically
than metal, but still wasn't amazing. If I ever rebuild the lathe spindle,
I'll replace the gear with another timing pulley.

## Controller assembly

`lathe_els_panel.FCStd` contains the CAD for the controller assembly, which
consists of a front panel in 2mm sheet metal, a protective lens machined from
2mm acrylic and a bunch of off-the-shelf standoffs and fasteners.


