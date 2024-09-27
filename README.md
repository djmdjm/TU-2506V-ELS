# Electronic Leadscrew for Optimum TU-2506V

This is a project to build an Electronic Leadscrew (ELS) for my bench
lathe, an Optimum TU-2506V. There are other lathes on the market that
seem to be based on the same casting (e.g. the Weiss WBL250F) that
might be compatible.

![installed ELS controller](/images/installed.jpg)

The project contains CAD designs for the mechanical parts, schematics,
PCB designs and firmware for the control board.

## Caveat

This works fine for me, but I'm just some guy on the internet with
little or no mechanical, electronic or software engineering training. 
There is no warranty for any of this, no guarantee that it will work,
no guarantee that it won't destroy your lathe, no promise that the
software won't crash in the middle of a job and scrap your part, etc.

Many of the mechanical pieces were manually made and even the
ones I CNC'd or had fabricated received some modifications that
aren't reflected in the CAD files. If you want to use this as the
basis of your own ELS, then be prepared to do some modifications of
your own.

## What you'll need (software)

The mechanical CAD designs were done in
[FreeCAD](https://www.freecad.org/) 0.21. For the parts that I CNC'd
myself, I also used FreeCAD's CAM processor to generate the G-Code.

The control boards was designed in [KiCAD](https://www.kicad.org/)
version 8 and were fabricated at JLCPCB. The PCB files has LCSC
annotations for automated assembly of the (few) SMT components.

The firmware for the control board was written in Rust using the
[stm32f4xx_hal](https://github.com/stm32-rs/stm32f4xx-hal) package.
I used rustc 1.81.0, but other versions might work too.

## What you'll need (hardware)

This lists only the main components. There are many small parts that
should be obvious from the CAD files, not to mention custom parts that
you'll need to make yourself or have fabricated.

The controller was designed around a
[STM32F411 "black pill"](https://www.dfrobot.com/product-2338.html) board.
These are also easily and cheaply available from hobbyist electronics
suppliers and from Ebay/Aliexpress.

The display is a
[Newhaven M0216SD‐162SDAR2-1](https://au.mouser.com/ProductDetail/Newhaven-Display/M0216SD-162SDAR2-1?qs=3vk7fz9CmNxwBN2LYkSmDA%3D%3D)
vacuum fluorescent display that I ordered from Mouser - this is a
fairly expensive part, but I love the aesthetics of a VFD.

The rest of the BOM for the controller can be extracted from the
KiCAD files for each of the respective boards. I mostly used parts that
I had on hand, so there are some odd choices in places.

The servo motor I used a
[Teknic Clearpath CPM-SDSK-2331S-ELS](https://teknic.com/model-info/CPM-SDSK-2331S-ELS/).
Again, this is a fairly expensive choice and a cheaper stepper
motor + controller would probably do fine. The Clearpath drive does
have some very nice advantages over plain steppers or even cheap
closed-loop stepper drives. Whatever drive you choose, it obviously
needs to fit (NEMA 23 was close to the limit on my lathe) and you'll
need to provide a suitable power supply for it.

The spindle encoder I used was an
[Omron E6B2-CWZ1X](https://www.ia.omron.com/product/item/2382/). These are
ridiculously expensive if you buy them from anywhere reputable, but very
cheap on Ebay or Aliexpress.

The timing pulleys and belt used to connect the servo to the
leadscrew are standard HTD 5M parts. I got mine from PT Parts
([pulleys](https://www.ptparts.com.au/products/category/ALL/-12-5M-15--5m-15-htd-timing-pulleys):
20-5M-15F 40-5M-15F,
[belt](https://www.ptparts.com.au/products/category/ALL/-225-5M-15--5m-15-timing-belts): 350-5M-15),
but they should be fairly standard.
