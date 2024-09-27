//! Fast GPIO pulse generator.
use core::arch::asm;
use stm32f4xx_hal::gpio;
use stm32f4xx_hal::gpio::PinExt;
use stm32f4xx_hal::pac;

pub struct Pulser<'a> {
    set_addr: usize,
    reset_addr: usize,
    _pin: &'a mut gpio::PEPin<'B', gpio::Output>,
}

impl<'a> Pulser<'a> {
    pub fn new(pin: &'a mut gpio::PEPin<'B', gpio::Output>) -> Self {
        unsafe {
            let gpiob_bsrr = (*pac::GPIOB::ptr()).bsrr.as_ptr() as usize;
            let (set_addr, reset_addr) = Self::bitband_addrs(gpiob_bsrr, pin.pin_id());
            Pulser {
                set_addr,
                reset_addr,
                _pin: pin, // Hang on to pin for safety.
            }
        }
    }

    fn bitband_addrs(bsrr_addr: usize, bit: u8) -> (usize, usize) {
        const PERIPHERALS_ADDR: usize = 0x40000000;
        const BITBAND_ADDR: usize = 0x42000000;
        let base: usize = BITBAND_ADDR + ((bsrr_addr - PERIPHERALS_ADDR) * 32);
        (base + 4 * bit as usize, base + 4 * (bit as usize + 16))
    }

    pub fn pulse(&mut self, count: u32) {
        if count == 0 {
            return;
        }
        unsafe {
            asm!(
            "mov {one}, #1",
            "2:",       // main loop.
                "str {one}, [{set_addr}]",
                // fine high duty cycle tweaking.
                "nop; nop; nop; nop",
                "mov {i}, #5",
                "3:",   // loop to set high duty cycle.
                    "sub {i}, #1",
                    "cmp {i}, #0",
                    "bne 3b",
                "str {one}, [{reset_addr}]",
                // fine low duty cycle tweaking.
                "nop; nop; nop",
                "mov {i}, #5",
                "4:",   // loop to set pulse low duty cycle.
                    "sub {i}, #1",
                    "cmp {i}, #0",
                    "bne 4b",
                "sub {count}, #1",
                // XXX would rather use cbnz but got compiler errors.
                "cmp {count}, #0",
                "bne 2b",
                    count = in(reg) count,
                    set_addr = in(reg) self.set_addr,
                    reset_addr = in(reg) self.reset_addr,
                    one = out(reg) _,
                    i = out(reg) _,
                    options(nostack),
                );
        }
    }
}
