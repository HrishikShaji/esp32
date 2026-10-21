use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{self};
use esp_idf_hal::gpio::{Output, PinDriver};
use esp_idf_hal::ledc::LedcDriver;
use esp_idf_svc::http::server::EspHttpServer;

fn delay(s: u64) {
    std::thread::sleep(Duration::from_secs(s));
}

pub fn init_server(
    led_pin: Arc<Mutex<PinDriver<'static, Output>>>,
    servo1: Arc<Mutex<LedcDriver<'static>>>,
    servo2: Arc<Mutex<LedcDriver<'static>>>,
) -> anyhow::Result<EspHttpServer<'static>, anyhow::Error> {
    let mut server = EspHttpServer::new(&Default::default()).unwrap();

    let led_pin_root = Arc::clone(&led_pin);

    server.fn_handler(
        "/",
        embedded_svc::http::Method::Get,
        move |req| -> anyhow::Result<()> {
            let mut response = req.into_ok_response()?;
            response.write("hello from esp32".as_bytes())?;
            let _led_root = led_pin_root.lock().unwrap().toggle();
            Ok(())
        },
    )?;

    server.fn_handler(
        "/led",
        embedded_svc::http::Method::Post,
        move |mut req| -> anyhow::Result<()> {
            let mut buffer = [0u8; 1024];

            let len = req.read(&mut buffer)?;

            let received = std::str::from_utf8(&buffer[..len])?.trim();

            let number: u64 = received.parse()?;

            println!("number:{:?}", number);

            let mut response = req.into_ok_response()?;
            response.write("This is led route".as_bytes())?;

            for i in 0..number {
                println!("Iteration:{}", i);
                let _led_on = led_pin.lock().unwrap().set_high();
                delay(1);
                let _led_off = led_pin.lock().unwrap().set_low();
                delay(1);
            }

            Ok(())
        },
    )?;

    fn interpolate(angle: u32, min: u32, max: u32) -> u32 {
        angle * (max - min) / 180 + min
    }

    fn rotate_servo(servo: Arc<Mutex<LedcDriver<'static>>>, angle: u32) {
        let max_duty = servo.lock().unwrap().get_max_duty();

        let min = max_duty / 40;
        let max = max_duty / 8;

        servo
            .lock()
            .unwrap()
            .set_duty(interpolate(angle, min, max))
            .unwrap();
        // delay(1);
        // servo.lock().unwrap().disable().unwrap();
        // delay(1);
    }

    let servo_for_single = Arc::clone(&servo1);

    server.fn_handler(
        "/servo",
        embedded_svc::http::Method::Post,
        move |mut req| -> anyhow::Result<()> {
            let mut buffer = [0u8; 6];

            let bytes_read = req.read(&mut buffer)?;
            let angle_string = from_utf8(&buffer[..bytes_read])?;
            let angle: u32 = angle_string.parse()?;

            rotate_servo(Arc::clone(&servo_for_single), angle);

            Ok(())
        },
    )?;

    let servo_for_loop = Arc::clone(&servo1);

    server.fn_handler(
        "/servo-loop",
        embedded_svc::http::Method::Post,
        move |_req| -> anyhow::Result<()> {
            for i in 0..6 {
                rotate_servo(Arc::clone(&servo_for_loop), (i + 1) * 30);
                delay(1);
            }

            Ok(())
        },
    )?;

    let servo1_for_multi = Arc::clone(&servo1);
    let servo2_for_multi = Arc::clone(&servo2);

    server.fn_handler(
        "/servo-multi",
        embedded_svc::http::Method::Post,
        move |_req| -> anyhow::Result<()> {
            for i in 0..6 {
                rotate_servo(Arc::clone(&servo1_for_multi), (i + 1) * 30);
                delay(1);
                rotate_servo(Arc::clone(&servo2_for_multi), (i + 1) * 30);
                delay(1);
            }

            Ok(())
        },
    )?;

    let servo1_for_reset = Arc::clone(&servo1);
    let servo2_for_reset = Arc::clone(&servo2);
    server.fn_handler(
        "/servo-reset",
        embedded_svc::http::Method::Post,
        move |_req| -> anyhow::Result<()> {
            for _ in 0..6 {
                rotate_servo(Arc::clone(&servo1_for_reset), 0);
                delay(1);
                rotate_servo(Arc::clone(&servo2_for_reset), 0);
                delay(1);
            }

            Ok(())
        },
    )?;
    Ok(server)
}
