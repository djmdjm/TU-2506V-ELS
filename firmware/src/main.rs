#![no_std]
#![no_main]

//use panic_probe as _;
use panic_halt as _;
use stm32f4xx_hal as hal;

mod lcd;
use lcd::*;
mod control;
mod fir;
mod userinterface;
use userinterface::Mode;
mod pulse;
use pulse::Pulser;

use core::cell::{Cell, RefCell};
use core::fmt::Write;
//use cortex_m::asm::delay;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
//use cortex_m_semihosting::hprintln;
use hal::dwt::DwtExt;
use hal::gpio::Speed;
use hal::pac;
use hal::pac::interrupt;
use hal::prelude::*;
use hal::qei::Qei;
use hal::timer::{CounterUs, Event};

const ENCODER_PPR: i64 = 2000;
const ENCODER_RATIO_SPINDLE: i64 = -40;
const ENCODER_RATIO_ENCODER: i64 = 80;

const LEADSCREW_PITCH: i64 = 3000; // µm
const DRIVE_RATIO_MOTOR: i64 = -20;
const DRIVE_RATIO_LEADSCREW: i64 = 80; // 40 tooth pulley and x0.5 gearbox.

const MOTOR_PPR: i64 = 3200;

const UI_ENCODER_PULSE_PER_DETENT: u32 = 2;
const DISPLAY_UPDATE_RATE: u32 = 10; // Hz

const RPM_SMOOTH_UPDATE_RATE: u32 = 50; // Hz
const RPM_SMOOTH_DISPLAY_RATE: u32 = DISPLAY_UPDATE_RATE; // Hz
const RPM_SMOOTH_FIR_DEPTH: usize = 20;

type Fir = fir::FirFilter<RPM_SMOOTH_FIR_DEPTH>;
static G_RPM: Mutex<Cell<i32>> = Mutex::new(Cell::new(0));
static G_ENC: Mutex<Cell<i32>> = Mutex::new(Cell::new(0));
static G_NOW: Mutex<Cell<i64>> = Mutex::new(Cell::new(0));
static G_TIM: Mutex<RefCell<Option<CounterUs<pac::TIM5>>>> = Mutex::new(RefCell::new(None));

#[interrupt]
fn TIM5() {
    // Take static reference to counter.
    static mut TIM: Option<CounterUs<pac::TIM5>> = None;
    let tim = TIM.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| G_TIM.borrow(cs).replace(None).unwrap())
    });

    let mut ms: i64 = 0;
    static RPM_FIR: Mutex<RefCell<Fir>> = Mutex::new(RefCell::new(Fir::new()));
    cortex_m::interrupt::free(|cs| {
        // Update monotonic millisecond counter.
        let now = G_NOW.borrow(cs);
        // Yes, this will fail after a few thousand centuries of uptime.
        // I'll fix it closer to then.
        ms = now.get().saturating_add(1);
        now.set(ms);
        // Calculate a FIR-smoothed RPM value every RPM_SMOOTH_UPDATE_RATE Hz.
        if ms % (1000 / RPM_SMOOTH_UPDATE_RATE) as i64 == 0 {
            let enc = G_ENC.borrow(cs);
            RPM_FIR.borrow(cs).borrow_mut().update(enc.get());
            enc.set(0);
        }
        if ms % (1000 / RPM_SMOOTH_DISPLAY_RATE) as i64 == 0 {
            let mut val = RPM_FIR.borrow(cs).borrow().filtered_value() as i64;
            // 'val' is average encoder pulses per interval over the FIR
            // period; convert this to RPM.
            val *= RPM_SMOOTH_UPDATE_RATE as i64; // pulses per second.
            val *= 60; // pulses per minute.
            val = (val * ENCODER_RATIO_SPINDLE) / ENCODER_RATIO_ENCODER;
            val /= ENCODER_PPR; // RPM
            G_RPM.borrow(cs).set(val as i32);
        }
    });

    let _ = tim.wait();
}

#[entry]
fn main() -> ! {
    //hprintln!("start");
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();

    let clocks = rcc
        .cfgr
        .use_hse(25.MHz())
        .sysclk(100.MHz())
        .hclk(25.MHz())
        .freeze();

    let dwt = cp.DWT.constrain(cp.DCB, &clocks);
    let mut ns_delay = dwt.delay();
    let mut timer = dp.TIM5.counter_us(&clocks);
    let mut delay = dp.TIM9.delay_us(&clocks);

    // Start the timer to give us a 1kHz clock interrupt.
    timer.start(1.millis()).unwrap();
    timer.listen(Event::Update);
    cortex_m::interrupt::free(|cs| *G_TIM.borrow(cs).borrow_mut() = Some(timer));
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM5);
    }

    // Encoders and associated switches.
    let spindle_enc = Qei::new(dp.TIM2, (gpioa.pa0, gpioa.pa1));
    let feed_enc = Qei::new(dp.TIM3, (gpioa.pa6, gpioa.pa7));
    let mode_enc = Qei::new(dp.TIM4, (gpiob.pb6, gpiob.pb7));
    let feed_enc_sw = gpioc.pc15.into_input();
    let mode_enc_sw = gpiob.pb8.into_input();

    // Control buttons.
    let button1 = gpioa.pa4.into_input();

    // Optocoupled inputs.
    let servo_ok_in = gpioa.pa3.into_input();

    // LED outputs.
    let mut board_led = gpioc.pc13.into_push_pull_output();

    // Optocoupled outputs.
    let mut motor_enable_out = gpiob.pb10.into_push_pull_output();
    let mut motor_dir_out = gpiob.pb1.into_push_pull_output();
    let mut motor_step_out = gpiob.pb0.into_push_pull_output().erase_number();

    // Display I/O.
    let mut disp_rs = gpiob.pb3.into_push_pull_output().speed(Speed::Medium);
    let mut disp_rw = gpiob.pb9.into_push_pull_output().speed(Speed::Medium);
    let mut disp_e = gpiob.pb4.into_push_pull_output().speed(Speed::Medium);
    let mut disp_db0 = gpioa.pa15.into_push_pull_output().speed(Speed::Medium);
    let mut disp_db1 = gpioa.pa12.into_push_pull_output().speed(Speed::Medium);
    let mut disp_db2 = gpioa.pa8.into_push_pull_output().speed(Speed::Medium);
    let mut disp_db3 = gpiob.pb14.into_push_pull_output().speed(Speed::Medium);
    let mut disp_db4 = gpiob.pb13.into_push_pull_output().speed(Speed::Medium);
    let mut disp_db5 = gpiob.pb12.into_push_pull_output().speed(Speed::Medium);
    let mut disp_db6 = gpiob.pb15.into_push_pull_output().speed(Speed::Medium);
    let mut disp_db7 = gpioa.pa11.into_push_pull_output().speed(Speed::Medium);
    let mut display = lcd::Display8Bit::new(
        &mut disp_rs,
        &mut disp_rw,
        &mut disp_e,
        &mut disp_db0,
        &mut disp_db1,
        &mut disp_db2,
        &mut disp_db3,
        &mut disp_db4,
        &mut disp_db5,
        &mut disp_db6,
        &mut disp_db7,
        &mut ns_delay,
    );
    display.init();
    write!(display.at(6, 0), "hello").ok();
    write!(display.at(6, 1), "there!").ok();
    delay.delay_ms(500u32);
    display.clear();
    motor_dir_out.set_low();
    motor_enable_out.set_low();
    motor_step_out.set_low();
    let mut pulser = Pulser::new(&mut motor_step_out);
    let mut cold: bool = true;
    let mut last_ms: i64 = 0;
    let mut now_ms: i64 = 0;
    let mut next_ui_ms: i64 = 0;
    let mut spindle_enc_count = spindle_enc.count() as i32;
    let mut spindle_enc_last = spindle_enc_count;
    let mut spindle_enc_delta: i32 = 0;
    let mut smoothed_rpm: i32 = 0;
    let mut ui = userinterface::UI::new(&mut display);
    let mut control = control::Control::new();
    let mut last_motor_dir: bool = false;
    let mut last_motor_enable: bool = false;
    let mut motor_pulses_since_last_ui: u32 = 0;
    loop {
        // Twiddle board LED as heartbeat.
        board_led.set_state((now_ms % 200 < 100).into());

        cortex_m::interrupt::free(|cs| {
            // Calculate number/direction of spindle encoder pulses.
            spindle_enc_count = spindle_enc.count() as i32;
            spindle_enc_delta = spindle_enc_count - spindle_enc_last;
            spindle_enc_last = spindle_enc_count;
            // Update global encoder pulse accumulator.
            let enc = G_ENC.borrow(cs);
            enc.set(enc.get() + spindle_enc_delta);
            // Get global millisecond counter and smoothed RPM value.
            now_ms = G_NOW.borrow(cs).get();
            smoothed_rpm = G_RPM.borrow(cs).get();
        });

        // Calculate elapsed ms since last loop iteration.
        let mut ms_elapsed: u32 = 0;
        if cold {
            cold = false;
        } else {
            ms_elapsed = (now_ms - last_ms).try_into().unwrap_or(0);
        }
        last_ms = now_ms;

        // Servo OK input is inverted.
        let servo_ok: bool = servo_ok_in.is_low();

        if next_ui_ms < now_ms {
            ui.update(
                &mut control,
                now_ms,
                smoothed_rpm,
                servo_ok,
                button1.is_high(),
                mode_enc.count() as i16,
                mode_enc_sw.is_low(),
                feed_enc.count() as i16,
                feed_enc_sw.is_low(),
                spindle_enc_count,
                motor_pulses_since_last_ui,
                last_motor_enable,
                last_motor_dir,
            );
            next_ui_ms = now_ms + (1000 / DISPLAY_UPDATE_RATE as i64);
            motor_pulses_since_last_ui = 0;
        }
        // Call controller for current mode to determine motor commands.
        let mut motor_enable: bool = false;
        let mut motor_dir: bool = false;
        let mut motor_pulses: u32 = 0;
        match ui.get_mode() {
            Mode::ServoOff => (),
            Mode::Feed | Mode::ThreadMetric | Mode::ThreadImperial => {
                // Prepare to command drive based on spindle motion and/or time.
                let (direction, pulses) = control.feed_per_rev(spindle_enc_delta, ms_elapsed);
                motor_dir = direction.into();
                motor_pulses = pulses;
                motor_enable = true;
            }
        }
        // Changes in enable and direction require at least 1μs to
        // be recognised.
        if !last_motor_enable {
            motor_enable_out.set_state(motor_enable.into());
            delay.delay_us(2);
            last_motor_enable = motor_enable;
        }
        // Don't bother sending pulses if the drive has alarmed or is disabled.
        if !servo_ok || !motor_enable {
            continue;
        }
        if motor_dir != last_motor_dir {
            motor_dir_out.set_state(motor_dir.into());
            delay.delay_us(2);
            last_motor_dir = motor_dir;
        }
        // XXX warning if we're not keeping up.
        pulser.pulse(motor_pulses);
        motor_pulses_since_last_ui += motor_pulses;
    }
}
