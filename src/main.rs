mod components;
mod connections;
mod servers;

use crate::components::led_matrix::{Max7219, NUM};
use crate::components::oled::{initiate_display, render_text};
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
    // let _display = init_display(sda, scl, i2c_driver);
    // let _display = display_text(sda, scl, i2c_driver);
    let mut display = initiate_display(sda, scl, i2c_driver);
    // render_text(&mut display,"hello");

    println!("OLED should now be illuminated.");

    //after display
    // log::info!("Hello,final sample world!");
    // let  led_pin = Arc::new(Mutex::new(PinDriver::output(peripherals.pins.gpio5).unwrap()));
    //
    // let servo1_timer = peripherals.ledc.timer1;
    // let servo1_driver = LedcTimerDriver::new(servo1_timer, &TimerConfig::new().frequency(Hertz(50)).resolution(esp_idf_hal::ledc::Resolution::Bits14)).unwrap();
    //
    // let servo1 = Arc::new(Mutex::new(LedcDriver::new(peripherals.ledc.channel3, servo1_driver, peripherals.pins.gpio2).unwrap()));
    //
    // let servo2_timer = peripherals.ledc.timer2;
    // let servo2_driver = LedcTimerDriver::new(servo2_timer, &TimerConfig::new().frequency(Hertz(50)).resolution(esp_idf_hal::ledc::Resolution::Bits14)).unwrap();
    //
    // let servo2 = Arc::new(Mutex::new(LedcDriver::new(peripherals.ledc.channel4, servo2_driver, peripherals.pins.gpio18).unwrap()));
    //
    // let _wifi = simple_connect_wifi(peripherals.modem);
    // let _server = init_server(led_pin,servo1,servo2);

    let din = PinDriver::output(peripherals.pins.gpio23).unwrap();
    let clk = PinDriver::output(peripherals.pins.gpio18).unwrap();
    let cs = PinDriver::output(peripherals.pins.gpio5).unwrap();

    let mut disp = Max7219::new(din, clk, cs);
    disp.init();

    // let mut led = PinDriver::output(peripherals.pins.gpio2).unwrap();

    // let button_up = PinDriver::input(peripherals.pins.gpio4, Pull::Up).unwrap();
    // let mut button_up_pressed = false;
    //
    // let button_left = PinDriver::input(peripherals.pins.gpio15, Pull::Up).unwrap();
    // let mut button_left_pressed = false;
    //
    // let button_right = PinDriver::input(peripherals.pins.gpio16, Pull::Up).unwrap();
    // let mut button_right_pressed = false;

    fn on_button_click(
        button_pin: &PinDriver<'static, Input>,
        button_pressed: &mut bool,
        mut action: impl FnMut(),
    ) {
        let pressed = button_pin.is_low();

        // if pressed != *button_pressed {
        //     println!(
        //         "{:?}: {}",
        //         match button_id {
        //             ButtonID::Up => "UP",
        //             ButtonID::Down => "DOWN",
        //             ButtonID::Left => "LEFT",
        //             ButtonID::Right => "RIGHT",
        //         },
        //         if pressed { "PRESSED" } else { "RELEASED" }
        //     );
        // }

        if pressed && !*button_pressed {
            action();
            // match button_id {
            //     ButtonID::Up => move_up(),
            //     ButtonID::Down => move_down(),
            //     ButtonID::Left => move_left(),
            //     ButtonID::Right => move_right(),
            // }
        }

        *button_pressed = pressed;
    }

    // led.set_low().unwrap();

    let mut joystick_button_pressed = false;

    // Demo: a vertical bar sweeping across all 32 columns
    let mut buf = [[0u8; 8]; NUM];
    let mut col = 0;
    let mut row = 0;
    let mut last_step = Instant::now();
    const STEP: Duration = Duration::from_millis(60);
    let mut led_on = false;
    let mut last_drawn: Option<bool> = None; // before the loop

    let mut current_row = 0;
    let mut current_bit = 0;
    loop {
        // Animation: advance one column every 60 ms, without blocking
        // if last_step.elapsed() >= STEP {
        //     last_step = Instant::now();
        //
        //     buf = [[0u8; 8]; NUM];
        //     let (m, bit) = (NUM - 1 - col / 8, 7 - (col % 8));
        //     for row in 0..8 {
        //         buf[m][row] = 1 << bit;
        //     }
        //     disp.show(&buf);
        //
        //     col = (col + 1) % (NUM * 8);
        // }

        // let bit = 5;
        //        buf[2][1] = 1 << bit;
        //        disp.show(&buf);

        // if joystick_button.is_low() {
        //     println!("Joystick button pressed!");
        // } else {
        //     println!("Joystick button released");
        // }

        let joystick_pressed = joystick_button.is_low();
        if joystick_pressed && !joystick_button_pressed {
            println!("joystick clicked");

            // current_bit = current_bit + 1;
            // let bit_mask = 1u8 << current_bit;
            // disp.move_down();
            // disp.show_buf();
            disp.update_buf_move_down();
            // update_row(&mut row, col, &mut buf, &mut disp);
            // led.toggle().unwrap();
            // led_on = !led_on;
        }
        joystick_button_pressed = joystick_pressed;

        // on_button_click(&button_up, &mut button_up_pressed, || {
        //     disp.update_buf_move_up()
        // });
        // on_button_click(&button_left, &mut button_left_pressed, || {
        //     disp.update_buf_move_left()
        // });
        // on_button_click(&button_right, &mut button_right_pressed, || {
        //     disp.update_buf_move_right()
        // });

        // let pressed = button_up.is_low();

        // Act only on the press edge
        // if pressed && !button_up_pressed {
        //     println!("button clicked");
        //     // update_col(&mut col,row,&mut buf,&mut disp);
        //     current_row = current_row + 1;
        //     disp.move_left(current_row);
        //     // disp.move_left(3);
        //     // disp.move_left(4);
        //     // disp.move_left(5);
        //     led.toggle().unwrap();
        //     led_on = !led_on;
        // }
        // button_up_pressed = pressed;

        // Redraw only when the LED state changes
        if last_drawn != Some(led_on) {
            // render_text(&mut display, if button_up_pressed { "on" } else { "off" });
            last_drawn = Some(led_on);
        }
        FreeRtos::delay_ms(1);
    }
    // loop {
    //     std::thread::sleep(Duration::from_secs(1));
    // }

    // Ok(())
}
