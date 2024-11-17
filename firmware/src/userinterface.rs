//! User interface code
use crate::control::{Control, Direction};
use crate::lcd;

const WELCOME_MESSAGE_TIMEOUT: i64 = 2500; // ms.
const WARN_MESSAGE_TIMEOUT: i64 = 500; // ms.
const BUTTON_HOLD_DEBUG_TIME: i64 = 1000; // ms.

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    ServoOff = 0,
    Feed,
    ThreadMetric,
    ThreadImperial,
}

impl Mode {
    pub fn add(&self, n: i32) -> Mode {
        match ((*self as i32) + n).clamp(0, 3) {
            0 => Mode::ServoOff,
            1 => Mode::Feed,
            2 => Mode::ThreadMetric,
            3 => Mode::ThreadImperial,
            _ => panic!(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum DebugPage {
    Help = 0,
    Status,
    UIControls,
    Motor,
    Spindle,
    Control,
    Time,
}

impl DebugPage {
    pub fn add(&self, n: i16) -> DebugPage {
        match ((*self as i16) + n).clamp(0, 6) {
            0 => DebugPage::Help,
            1 => DebugPage::Status,
            2 => DebugPage::UIControls,
            3 => DebugPage::Motor,
            4 => DebugPage::Spindle,
            5 => DebugPage::Control,
            6 => DebugPage::Time,
            _ => panic!(),
        }
    }
}

pub struct UI<'a, DISPLAY> {
    display: &'a mut DISPLAY,
    mode: Mode,
    last_update_ms: i64,
    debug_mode: bool,
    debug_page: DebugPage,
    message1: &'a str,
    message2: &'a str,
    message_timeout: i64,
    mode_enc_pos_last: i16,
    feed_enc_pos_last: i16,
    feed_rate_index: usize,
    metric_thread_pitch_index: usize,
    imperial_thread_pitch_index: usize,
    debug_hold: i64,
    spindle_enc_last: i32,
    cold: bool,
}

impl<'a, DISPLAY> UI<'a, DISPLAY>
where
    DISPLAY: lcd::CharacterDisplay + core::fmt::Write,
{
    const FEED_RATES: [i32; 58] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55, 60, 65, 70, 75, 80,
        85, 90, 95, 100, 110, 120, 130, 140, 150, 160, 170, 180, 190, 200, 225, 250, 270, 300, 375,
        400, 425, 450, 475, 500, 550, 600, 650, 700, 750, 800, 850, 900, 1000,
    ];
    const DEFAULT_FEED_RATE_INDEX: usize = 24;
    const METRIC_THREAD_PITCHES: [i32; 20] = [
        200, 250, 300, 350, 400, 450, 500, 600, 700, 750, 800, 1000, 1250, 1500, 1750, 2000, 2500,
        3000, 3500, 4000,
    ];
    const DEFAULT_METRIC_THREAD_PITCH: usize = 11;
    // XXX need more accuracy here. Switch to nanometres/rev?
    const IMPERIAL_THREAD_PITCHES: [i32; 21] = [
        80, 72, 64, 56, 48, 40, 32, 28, 24, 20, 18, 16, 14, 13, 12, 11, 10, 9, 8, 7, 6,
    ];
    const DEFAULT_IMPERIAL_THREAD_PITCH: usize = 9;
    pub fn new(display: &'a mut DISPLAY) -> UI<'a, DISPLAY> {
        UI {
            display,
            mode: Mode::ServoOff,
            last_update_ms: 0,
            debug_mode: false,
            debug_page: DebugPage::Help,
            message1: "",
            message2: "",
            message_timeout: -1,
            mode_enc_pos_last: 0,
            feed_enc_pos_last: 0,
            feed_rate_index: Self::DEFAULT_FEED_RATE_INDEX,
            metric_thread_pitch_index: Self::DEFAULT_METRIC_THREAD_PITCH,
            imperial_thread_pitch_index: Self::DEFAULT_IMPERIAL_THREAD_PITCH,
            debug_hold: 0,
            spindle_enc_last: 0,
            cold: true,
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        control: &mut Control,
        now_ms: i64,
        rpm: i32,
        servo_ok: bool,
        button1: bool,
        mode_enc_pos: i16,
        mode_enc_button: bool,
        feed_enc_pos: i16,
        feed_enc_button: bool,
        spindle_enc_pos: i32,
        motor_pulses: u32,
        motor_enable: bool,
        motor_direction: bool,
    ) {
        if self.cold {
            self.last_update_ms = now_ms;
            self.mode_enc_pos_last = mode_enc_pos;
            self.feed_enc_pos_last = feed_enc_pos;
            control.set_feed_rate_micron_per_rev(Self::FEED_RATES[self.feed_rate_index]);
            self.message1 = "TU-2506V-ELS";
            self.message2 = "djm 20241117";
            self.message_timeout = now_ms + WELCOME_MESSAGE_TIMEOUT;
            self.cold = false;
        }
        let mode_enc_pulses: i16 =
            (mode_enc_pos - self.mode_enc_pos_last) / crate::UI_ENCODER_PULSE_PER_DETENT as i16;
        let feed_enc_pulses: i16 =
            (feed_enc_pos - self.feed_enc_pos_last) / crate::UI_ENCODER_PULSE_PER_DETENT as i16;
        self.mode_enc_pos_last = mode_enc_pos;
        self.feed_enc_pos_last = feed_enc_pos;
        let spindle_moving = rpm > 3;

        let mut status: &str = "OK";
        if self.mode == Mode::ServoOff {
            status = "OFF";
        } else if !servo_ok {
            status = "!SERVO";
        }

        // Hold mode+feed encoder buttons down to toggle debug display.
        // While button is held, ignore rotation of the encoder.
        if mode_enc_button && feed_enc_button {
            if self.debug_hold == 0 {
                // Just started pressing mode enc button for debug mode.
                // Start timeout.
                self.debug_hold = now_ms + BUTTON_HOLD_DEBUG_TIME;
            } else if self.debug_hold < now_ms {
                // Held for long enough; enter/exit debug display.
                self.debug_mode = !self.debug_mode;
                self.debug_page = DebugPage::Help;
                self.debug_hold = 0;
            }
        } else {
            self.debug_hold = 0;
        }

        if mode_enc_pulses != 0 && self.debug_hold == 0 {
            if self.debug_mode {
                self.debug_page = self.debug_page.add(mode_enc_pulses);
            } else if spindle_moving {
                // Don't allow mode changes while the spindle is running.
                self.message1 = "STOP SPINDLE";
                self.message2 = "TO CHANGE MODE";
                self.message_timeout = now_ms + WARN_MESSAGE_TIMEOUT;
            } else if !mode_enc_button {
                // Require button to be pressed too.
                self.message1 = "PRESS KNOB TO";
                self.message2 = "CHANGE MODE";
                self.message_timeout = now_ms + WARN_MESSAGE_TIMEOUT;
            }
        }

        // Special handling for threading modes: don't allow pitch changes
        // while spindle is moving. Also require button be pressed.
        if !self.debug_mode && feed_enc_pulses != 0 && self.debug_hold == 0 {
            match self.mode {
                Mode::ThreadMetric | Mode::ThreadImperial => {
                    if spindle_moving {
                        self.message1 = "STOP SPINDLE TO";
                        self.message2 = "CHANGE PITCH";
                        self.message_timeout = now_ms + WARN_MESSAGE_TIMEOUT;
                    } else if !feed_enc_button {
                        self.message1 = "PRESS KNOB TO";
                        self.message2 = "CHANGE PITCH";
                        self.message_timeout = now_ms + WARN_MESSAGE_TIMEOUT;
                    }
                }
                _ => (),
            }
        }

        // If there's an active warning message then display that.
        if self.message_timeout > now_ms {
            write!(self.display.at(0, 0), "{:^16}", self.message1).ok();
            write!(self.display.at(0, 1), "{:^16}", self.message2).ok();
            return;
        }

        // If spindle stopped and mode wheel moved, then change mode.
        let mut mode_changed: bool = false;
        if !self.debug_mode && self.debug_hold == 0 && mode_enc_pulses != 0 {
            let new_mode: Mode = self.mode.add(mode_enc_pulses.into());
            mode_changed = new_mode != self.mode;
            self.mode = new_mode;
        }

        // If feed changed, update (unless in debug mode).
        if !self.debug_mode && self.debug_hold == 0 && (mode_changed || feed_enc_pulses != 0) {
            match self.mode {
                Mode::Feed => self.update_feed(control, feed_enc_pulses),
                Mode::ThreadMetric => self.update_thread_metric(control, feed_enc_pulses),
                Mode::ThreadImperial => self.update_thread_imperial(control, feed_enc_pulses),
                Mode::ServoOff => (),
            }
        }

        // Update display.
        if self.debug_mode {
            self.display_debug(
                control,
                now_ms,
                rpm,
                status,
                mode_enc_pos,
                mode_enc_button,
                feed_enc_pos,
                feed_enc_button,
                button1,
                spindle_enc_pos,
                motor_pulses,
                motor_enable,
                motor_direction,
                servo_ok,
            );
            self.spindle_enc_last = spindle_enc_pos;
        } else {
            match self.mode {
                Mode::ServoOff => self.display_servo_off(rpm, status),
                Mode::Feed => self.display_feed(rpm, status),
                Mode::ThreadMetric => self.display_thread_metric(rpm, status),
                Mode::ThreadImperial => self.display_thread_imperial(rpm, status),
            }
        }
        self.last_update_ms = now_ms;
    }
    pub fn get_mode(&self) -> Mode {
        self.mode
    }

    // Update feed mode parameters based on user input.
    fn update_feed(&mut self, control: &mut Control, feed_enc_pulses: i16) {
        self.feed_rate_index = (self.feed_rate_index as isize + feed_enc_pulses as isize)
            .clamp(0, Self::FEED_RATES.len() as isize - 1) as usize;
        control.set_feed_rate_micron_per_rev(Self::FEED_RATES[self.feed_rate_index])
    }

    // Update metric thread mode parameters based on user input.
    fn update_thread_metric(&mut self, control: &mut Control, feed_enc_pulses: i16) {
        self.metric_thread_pitch_index =
            (self.metric_thread_pitch_index as isize + feed_enc_pulses as isize)
                .clamp(0, Self::METRIC_THREAD_PITCHES.len() as isize - 1) as usize;
        control.set_feed_rate_micron_per_rev(
            Self::METRIC_THREAD_PITCHES[self.metric_thread_pitch_index],
        )
    }

    // Update imperial thread mode parameters based on user input.
    fn update_thread_imperial(&mut self, control: &mut Control, feed_enc_pulses: i16) {
        self.imperial_thread_pitch_index =
            (self.imperial_thread_pitch_index as isize + feed_enc_pulses as isize)
                .clamp(0, Self::IMPERIAL_THREAD_PITCHES.len() as isize - 1) as usize;
        let tpi = Self::IMPERIAL_THREAD_PITCHES[self.imperial_thread_pitch_index];
        control.set_feed_rate_tpi(tpi);
    }

    // Display for servo off mode.
    fn display_servo_off(&mut self, rpm: i32, status: &str) {
        write!(self.display.at(0, 0), "{:<16}", "Servo off").ok();
        write!(self.display.at(0, 1), "RPM {:<+5} {:>6}", rpm, status).ok();
    }

    // Display for feed mode.
    fn display_feed(&mut self, rpm: i32, status: &str) {
        write!(
            self.display.at(0, 0),
            "Feed {:>+7}μm/r",
            Self::FEED_RATES[self.feed_rate_index]
        )
        .ok();
        write!(self.display.at(0, 1), "RPM {:<+5} {:>6}", rpm, status).ok();
    }

    // Display for metric thread mode.
    fn display_thread_metric(&mut self, rpm: i32, status: &str) {
        let pitch = Self::METRIC_THREAD_PITCHES[self.metric_thread_pitch_index];
        let whole = pitch / 1000;
        let frac = (pitch.abs() / 10) % 100;
        write!(self.display.at(0, 0), "Thread{:>+3}.{:02}mm/r", whole, frac).ok();
        write!(self.display.at(0, 1), "RPM {:<+5} {:>6}", rpm, status).ok();
    }

    // Display for imperial thread mode.
    fn display_thread_imperial(&mut self, rpm: i32, status: &str) {
        let tpi = Self::IMPERIAL_THREAD_PITCHES[self.imperial_thread_pitch_index];
        write!(self.display.at(0, 0), "Thread Im {:>+3}TPI", tpi).ok();
        write!(self.display.at(0, 1), "RPM {:<+5} {:>6}", rpm, status).ok();
    }

    fn onoff(v: bool) -> char {
        if v {
            '●'
        } else {
            '○'
        }
    }

    // Debug display.
    #[allow(clippy::too_many_arguments)]
    fn display_debug(
        &mut self,
        control: &mut Control,
        now_ms: i64,
        rpm: i32,
        status: &str,
        mode_enc_pos: i16,
        mode_enc_button: bool,
        feed_enc_pos: i16,
        feed_enc_button: bool,
        button1: bool,
        spindle_encoder_pos: i32,
        motor_pulses: u32,
        motor_enable: bool,
        motor_direction: bool,
        servo_ok: bool,
    ) {
        match self.debug_page {
            DebugPage::Help => {
                write!(self.display.at(0, 0), "{:<16}", "Debug0: help").ok();
                write!(self.display.at(0, 1), "{:<16}", "↑↓ w/ mode dial").ok();
            }
            DebugPage::Status => {
                write!(self.display.at(0, 0), "{:<16}", "Debug1: Status").ok();
                write!(self.display.at(0, 1), "RPM {:<+5} {:>6}", rpm, status).ok();
            }
            DebugPage::UIControls => {
                write!(self.display.at(0, 0), "{:<16}", "Debug2: UI input").ok();
                write!(
                    self.display.at(0, 1),
                    "{}{}{:<+6} {}{:<+6}",
                    Self::onoff(button1),
                    Self::onoff(mode_enc_button),
                    mode_enc_pos,
                    Self::onoff(feed_enc_button),
                    feed_enc_pos
                )
                .ok();
            }
            DebugPage::Motor => {
                write!(self.display.at(0, 0), "{:<16}", "Debug3: motor").ok();
                write!(
                    self.display.at(0, 1),
                    "{:>3} {:>3} {}{:>7}",
                    if servo_ok { "OK" } else { "ERR" },
                    if motor_enable { "EN" } else { "DIS" },
                    if motor_direction { "+" } else { "-" },
                    motor_pulses,
                )
                .ok();
            }
            DebugPage::Spindle => {
                write!(self.display.at(0, 0), "{:<16}", "Debug4: spindle").ok();
                write!(
                    self.display.at(0, 1),
                    "{:>08x} {:>+7}",
                    spindle_encoder_pos,
                    spindle_encoder_pos - self.spindle_enc_last,
                )
                .ok();
            }
            DebugPage::Control => {
                let mut feed = control.get_feed_rate_micron_per_rev();
                match control.get_last_direction() {
                    Direction::Backwards => {
                        feed = -feed;
                    }
                    Direction::Forward => (),
                }

                write!(self.display.at(0, 0), "{:<16}", "Debug5: control").ok();
                write!(
                    self.display.at(0, 1),
                    "F{:>+5} R{:>+8}",
                    feed,
                    control.get_fractional_pulses_remaining(),
                )
                .ok();
            }
            DebugPage::Time => {
                write!(self.display.at(0, 0), "{:<16}", "Debug6: time").ok();
                write!(
                    self.display.at(0, 1),
                    "N{:>8x} L{:>5}",
                    now_ms,
                    now_ms - self.last_update_ms,
                )
                .ok();
            }
        }
    }
}
