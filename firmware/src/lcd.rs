//! Interface for HD44870-style character displays
use core::fmt::{self, Write};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

#[derive(Debug)]
pub enum Error {
    BoundsError,
    UnsupportedCharacter { c: char },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::BoundsError => write!(f, "Screen bounds exceeded"),
            Error::UnsupportedCharacter { c } => {
                write!(f, "display does not support character {}", c)
            }
        }
    }
}

// XXX want clear_to_eol() too. Maybe also program_character()

pub trait CharacterDisplay {
    fn init(&mut self);
    #[allow(dead_code)]
    fn cursor(&mut self, show_cursor: bool, blink_cursor: bool);
    #[allow(dead_code)]
    fn addr(&mut self, addr: u8);
    #[allow(dead_code)]
    fn char(&mut self, c: u8);
    #[allow(dead_code)]
    fn clear(&mut self);
    #[allow(dead_code)]
    fn position(&mut self, x: u8, y: u8) -> Result<(), Error>;
    #[allow(dead_code)]
    fn string(&mut self, s: &str) -> Result<u8, Error>;
    #[allow(dead_code)]
    fn at(&mut self, x: u8, y: u8) -> &mut Self;
}

pub struct Display8Bit<'a, RS, RW, E, DB0, DB1, DB2, DB3, DB4, DB5, DB6, DB7, DELAY> {
    rs: &'a mut RS,
    rw: &'a mut RW,
    e: &'a mut E,
    db0: &'a mut DB0,
    db1: &'a mut DB1,
    db2: &'a mut DB2,
    db3: &'a mut DB3,
    db4: &'a mut DB4,
    db5: &'a mut DB5,
    db6: &'a mut DB6,
    db7: &'a mut DB7,
    delay: &'a mut DELAY,
}
impl<'a, RS, RW, E, DB0, DB1, DB2, DB3, DB4, DB5, DB6, DB7, DELAY>
    Display8Bit<'a, RS, RW, E, DB0, DB1, DB2, DB3, DB4, DB5, DB6, DB7, DELAY>
where
    RS: OutputPin,
    RW: OutputPin,
    E: OutputPin,
    DB0: OutputPin,
    DB1: OutputPin,
    DB2: OutputPin,
    DB3: OutputPin,
    DB4: OutputPin,
    DB5: OutputPin,
    DB6: OutputPin,
    DB7: OutputPin,
    DELAY: DelayNs,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rs: &'a mut RS,
        rw: &'a mut RW,
        e: &'a mut E,
        db0: &'a mut DB0,
        db1: &'a mut DB1,
        db2: &'a mut DB2,
        db3: &'a mut DB3,
        db4: &'a mut DB4,
        db5: &'a mut DB5,
        db6: &'a mut DB6,
        db7: &'a mut DB7,
        delay: &'a mut DELAY,
    ) -> Display8Bit<'a, RS, RW, E, DB0, DB1, DB2, DB3, DB4, DB5, DB6, DB7, DELAY> {
        rs.set_high().ok();
        rw.set_high().ok();
        e.set_high().ok();
        db0.set_high().ok();
        db1.set_high().ok();
        db2.set_high().ok();
        db3.set_high().ok();
        db4.set_high().ok();
        db5.set_high().ok();
        db6.set_high().ok();
        db7.set_high().ok();
        Display8Bit {
            rs,
            rw,
            e,
            db0,
            db1,
            db2,
            db3,
            db4,
            db5,
            db6,
            db7,
            delay,
        }
    }
    fn write(&mut self, rs: bool, words: &[u8]) {
        //hprintln!("xxx write rs={} word={:?}", rs, words);
        self.rs.set_state(rs.into()).ok();
        for word in words {
            self.rw.set_low().ok();
            self.e.set_high().ok();
            // Tas = 20ns
            self.delay.delay_ns(20);
            self.db0.set_state((word & 0b00000001 != 0).into()).ok();
            self.db1.set_state((word & 0b00000010 != 0).into()).ok();
            self.db2.set_state((word & 0b00000100 != 0).into()).ok();
            self.db3.set_state((word & 0b00001000 != 0).into()).ok();
            self.db4.set_state((word & 0b00010000 != 0).into()).ok();
            self.db5.set_state((word & 0b00100000 != 0).into()).ok();
            self.db6.set_state((word & 0b01000000 != 0).into()).ok();
            self.db7.set_state((word & 0b10000000 != 0).into()).ok();
            // PWeh = 230ns
            self.delay.delay_ns(230);
            self.e.set_low().ok();
            // PWel = 230ns
            self.delay.delay_ns(230);
            self.e.set_high().ok();
            self.rw.set_high().ok();
        }
    }
}

#[allow(clippy::unusual_byte_groupings)]
impl<'a, RS, RW, E, DB0, DB1, DB2, DB3, DB4, DB5, DB6, DB7, DELAY> CharacterDisplay
    for Display8Bit<'a, RS, RW, E, DB0, DB1, DB2, DB3, DB4, DB5, DB6, DB7, DELAY>
where
    RS: OutputPin,
    RW: OutputPin,
    E: OutputPin,
    DB0: OutputPin,
    DB1: OutputPin,
    DB2: OutputPin,
    DB3: OutputPin,
    DB4: OutputPin,
    DB5: OutputPin,
    DB6: OutputPin,
    DB7: OutputPin,
    DELAY: DelayNs,
{
    fn init(&mut self) {
        let d: [u8; 4] = [0b00_111_0_00, 0b00001_100, 0b1_0000000, 0b00000001];
        self.write(false, &d);
    }
    fn cursor(&mut self, show_cursor: bool, blink_cursor: bool) {
        self.write(
            false,
            &[0b00001_100
                | (if show_cursor { 0x10 } else { 0b00 })
                | (if blink_cursor { 0b01 } else { 0b00 })],
        );
    }
    fn addr(&mut self, addr: u8) {
        //hprintln!("addr {:#x}", addr);
        self.write(false, &[0b1_0000000 | addr]);
    }
    fn char(&mut self, c: u8) {
        self.write(true, &[c]);
    }
    fn clear(&mut self) {
        self.write(false, &[0b00000001]);
    }
    fn position(&mut self, x: u8, y: u8) -> Result<(), Error> {
        if x > 16 || y > 1 {
            return Err(Error::BoundsError {});
        }
        //hprintln!("pos x={} y={}", x, y);
        self.addr(x | (y * 0x40));
        Ok(())
    }
    fn string(&mut self, s: &str) -> Result<u8, Error> {
        let mut d: [u8; 16] = [0u8; 16];
        let mut i: usize = 0;
        for c in s.chars() {
            if i >= d.len() {
                return Err(Error::BoundsError {});
            }
            d[i] = match c {
                '\\' => 0b10001100,
                '~' => 0b10001110,
                'Σ' => 0b11110110,
                '◀' => 0b00011110,
                '▲' => 0b00011111,
                '▶' => 0b00011101,
                '▼' => 0b00011100,
                '←' => 0b01111111,
                '↑' => 0b10011110,
                '→' => 0b01111110,
                '↓' => 0b10011111,
                '●' => 0b10010100,
                '°' => 0b11011111,
                '○' => 0b10010101,
                'α' => 0b11100000,
                'β' => 0b11100010,
                'θ' => 0b11110010,
                'μ' => 0b11100100,
                'π' => 0b11110111,
                'Ω' => 0b11110100,
                'ω' => 0b11110011,
                'ρ' => 0b11100110,
                'σ' => 0b11100101,
                'ε' => 0b11100011,
                ' '..='}' => c as u8,
                _ => return Err(Error::UnsupportedCharacter { c }),
            };
            i += 1;
        }
        self.write(true, &d[0..i]);
        Ok(i as u8)
    }
    fn at(&mut self, x: u8, y: u8) -> &mut Self {
        self.position(x, y).ok();
        self
    }
}

impl<'a, RS, RW, E, DB0, DB1, DB2, DB3, DB4, DB5, DB6, DB7, DELAY> Write
    for Display8Bit<'a, RS, RW, E, DB0, DB1, DB2, DB3, DB4, DB5, DB6, DB7, DELAY>
where
    RS: OutputPin,
    RW: OutputPin,
    E: OutputPin,
    DB0: OutputPin,
    DB1: OutputPin,
    DB2: OutputPin,
    DB3: OutputPin,
    DB4: OutputPin,
    DB5: OutputPin,
    DB6: OutputPin,
    DB7: OutputPin,
    DELAY: DelayNs,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.string(s).map_err(|_| fmt::Error)?;
        Ok(())
    }
}
