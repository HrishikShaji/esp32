mod components;
mod connections;
mod servers;

use crate::components::led_matrix::{Max7219, NUM};
use crate::components::oled::{initiate_display, render_text};
use esp_idf_hal::adc::attenuation::DB_12;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::{Input, Output, PinDriver, Pull};
use esp_idf_hal::peripherals::Peripherals;
use std::time::{Duration, Instant};

enum ButtonID {
    Up,
    Down,
    Left,
    Right,
}

fn main() {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    esp_idf_hal::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take().unwrap();

    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;
    let i2c_driver = peripherals.i2c0;

    let joystick_button = PinDriver::input(peripherals.pins.gpio19, Pull::Up).unwrap();

    // Initialize the joystick's analog inputs.
    // GPIO34 = X axis, GPIO35 = Y axis.
    let adc = AdcDriver::new(peripherals.adc1).unwrap();

    let adc_config = AdcChannelConfig {
        attenuation: DB_12,
        ..Default::default()
    };

    let mut joystick_x = AdcChannelDriver::new(&adc, peripherals.pins.gpio34, &adc_config).unwrap();
    let mut joystick_y = AdcChannelDriver::new(&adc, peripherals.pins.gpio35, &adc_config).unwrap();

    let mut display = initiate_display(sda, scl, i2c_driver);
    // render_text(&mut display, "hello");

    println!("OLED should now be illuminated.");

    // let led_pin = Arc::new(Mutex::new(PinDriver::output(peripherals.pins.gpio5).unwrap()));
    //
    // let servo1_timer = peripherals.ledc.timer1;
    // let servo1_driver = LedcTimerDriver::new(servo1_timer, &TimerConfig::new().frequency(Hertz(50)).resolution(esp_idf_hal::ledc::Resolution::Bits14)).unwrap();
    // let servo1 = Arc::new(Mutex::new(LedcDriver::new(peripherals.ledc.channel3, servo1_driver, peripherals.pins.gpio2).unwrap()));
    //
    // let servo2_timer = peripherals.ledc.timer2;
    // let servo2_driver = LedcTimerDriver::new(servo2_timer, &TimerConfig::new().frequency(Hertz(50)).resolution(esp_idf_hal::ledc::Resolution::Bits14)).unwrap();
    // let servo2 = Arc::new(Mutex::new(LedcDriver::new(peripherals.ledc.channel4, servo2_driver, peripherals.pins.gpio18).unwrap()));
    //
    // let _wifi = simple_connect_wifi(peripherals.modem);
    // let _server = init_server(led_pin, servo1, servo2);

    let din = PinDriver::output(peripherals.pins.gpio23).unwrap();
    let clk = PinDriver::output(peripherals.pins.gpio18).unwrap();
    let cs = PinDriver::output(peripherals.pins.gpio5).unwrap();

    let mut disp = Max7219::new(din, clk, cs);
    disp.init();

    let mut joystick_button_pressed = false;

    // ---------------------------------------------------------------
    // Joystick calibration
    // Keep the stick untouched (centered) while this runs at startup.
    // ---------------------------------------------------------------
    const SAMPLES: u32 = 64;
    let mut sum_x: u32 = 0;
    let mut sum_y: u32 = 0;
    for _ in 0..SAMPLES {
        sum_x += adc.read(&mut joystick_x).unwrap() as u32;
        sum_y += adc.read(&mut joystick_y).unwrap() as u32;
        FreeRtos::delay_ms(5);
    }
    let center_x = (sum_x / SAMPLES) as i32;
    let center_y = (sum_y / SAMPLES) as i32;
    println!("Calibrated center: x={} y={}", center_x, center_y);

    // How far from the center the stick must move to count as a direction.
    // Tune after checking the raw values (try 400..800).
    const THRESHOLD: i32 = 500;
    const MOVE_INTERVAL: Duration = Duration::from_millis(180);

    // Set to false once the joystick works, to stop the serial spam.
    const DEBUG_RAW: bool = true;
    let mut last_debug = Instant::now();

    let mut last_move = Instant::now();

    loop {
        // Read the joystick's analog axes.
        let x = adc.read(&mut joystick_x).unwrap() as i32;
        let y = adc.read(&mut joystick_y).unwrap() as i32;

        // Print raw values twice a second to help with tuning.
        if DEBUG_RAW && last_debug.elapsed() >= Duration::from_millis(500) {
            last_debug = Instant::now();
            println!("x={} y={}", x, y);
        }

        let dx = x - center_x;
        let dy = y - center_y;

        // Pick the axis with the larger deviation so the two axes don't mask each other.
        // If your directions come out swapped or inverted, swap Left/Right or Up/Down below.
        let direction = if dx.abs() < THRESHOLD && dy.abs() < THRESHOLD {
            None
        } else if dx.abs() >= dy.abs() {
            if dx < 0 {
                Some(ButtonID::Left)
            } else {
                Some(ButtonID::Right)
            }
        } else if dy < 0 {
            Some(ButtonID::Up)
        } else {
            Some(ButtonID::Down)
        };

        // Move once per interval while the joystick is held.
        if let Some(direction) = direction {
            if last_move.elapsed() >= MOVE_INTERVAL {
                last_move = Instant::now();

                match direction {
                    ButtonID::Up => {
                        println!("Joystick UP");
                        disp.update_buf_move_up();
                    }
                    ButtonID::Down => {
                        println!("Joystick DOWN");
                        disp.update_buf_move_down();
                    }
                    ButtonID::Left => {
                        println!("Joystick LEFT");
                        disp.update_buf_move_left();
                    }
                    ButtonID::Right => {
                        println!("Joystick RIGHT");
                        disp.update_buf_move_right();
                    }
                }
            }
        } else {
            // Allow an immediate movement when the joystick is pushed again.
            last_move = Instant::now()
                .checked_sub(MOVE_INTERVAL)
                .unwrap_or_else(Instant::now);
        }

        // Joystick click (act only on the press edge).
        let joystick_pressed = joystick_button.is_low();
        if joystick_pressed && !joystick_button_pressed {
            println!("joystick clicked");
            disp.update_buf_move_down();
        }
        joystick_button_pressed = joystick_pressed;

        FreeRtos::delay_ms(1);
    }
}
