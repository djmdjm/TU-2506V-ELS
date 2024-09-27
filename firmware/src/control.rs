//! Encoder/Servo control

#[derive(Clone, Copy)]
pub enum Direction {
    Forward,
    Backwards,
}

impl From<Direction> for bool {
    fn from(val: Direction) -> Self {
        match val {
            Direction::Forward => true,
            Direction::Backwards => false,
        }
    }
}

pub struct Control {
    // XXX direction?
    feed_rate_micron_per_rev: i32,

    feed_per_rev_factor: i64,
    fractional_pulses_remaining: i64,
    last_direction: Direction,
}

impl Control {
    pub fn new() -> Self {
        Control {
            feed_rate_micron_per_rev: 0,
            feed_per_rev_factor: 0,
            fractional_pulses_remaining: 0,
            last_direction: Direction::Forward,
        }
    }
    pub fn feed_per_rev(
        &mut self,
        encoder_pulses: i32,
        _elapsed_ms: u32,
    ) -> (Direction, u32) {
        let mut t: i64 = encoder_pulses as i64 * self.feed_per_rev_factor;
        t += self.fractional_pulses_remaining;
        // Retain remainder for next round.
        let mut pulses: i32 = (t >> 32) as i32;
        t -= (pulses as i64) << 32;
        self.fractional_pulses_remaining = t;
        let mut direction: Direction = self.last_direction;
        #[allow(clippy::comparison_chain)]
        if pulses > 0 {
            direction = Direction::Forward;
        } else if pulses < 0 {
            pulses = -pulses;
            direction = Direction::Backwards;
        }
        self.last_direction = direction;
        // XXX acceleration and deceleration control.
        // XXX hystereis for direction control.
        (direction, pulses as u32)
    }
    pub fn get_feed_rate_micron_per_rev(&self) -> i32 {
        self.feed_rate_micron_per_rev
    }
    pub fn get_last_direction(&self) -> Direction {
        self.last_direction
    }
    pub fn get_fractional_pulses_remaining(&self) -> i64 {
        self.fractional_pulses_remaining
    }
    pub fn set_feed_rate_micron_per_rev(&mut self, feed: i32) {
        // XXX bounds checking.
        self.feed_rate_micron_per_rev = feed;
        self.fractional_pulses_remaining = 0;
        // XXX consider fixed point split; is 32.32 ideal?
        // Precalculate multiplication factor.
        // Pulses to fractional turns (32.32 fixed point).
        let mut t: i64 = 1 << 32;
        t *= crate::ENCODER_RATIO_SPINDLE;
        t /= crate::ENCODER_PPR * crate::ENCODER_RATIO_ENCODER;
        // Encoder turns to fractional leadscrew turns (32.32).
        t *= self.feed_rate_micron_per_rev as i64;
        t *= crate::DRIVE_RATIO_LEADSCREW;
        t /= crate::DRIVE_RATIO_MOTOR * crate::LEADSCREW_PITCH;
        // Leadscrew turns to encoder pulses (32.32).
        t *= crate::MOTOR_PPR;
        self.feed_per_rev_factor = t
    }
}
