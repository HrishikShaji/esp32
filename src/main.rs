use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{self, Error};
use embedded_svc::wifi::{AuthMethod,ClientConfiguration,Configuration};
use esp_idf_hal::delay::BLOCK;
use esp_idf_hal::gpio::{Gpio5, Gpio21, Gpio22, Output, PinDriver};
use esp_idf_hal::i2c::{I2C0, I2cConfig, I2cDriver};
use esp_idf_hal::io::Read;
use esp_idf_hal::ledc::{LedcDriver, LedcTimerDriver};
use esp_idf_hal::ledc::config::TimerConfig;
use esp_idf_hal::modem::Modem;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::units::{Hertz, KiloHertz};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::http::server::EspHttpServer;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AsyncWifi, EspWifi};
use heapless::String;

const OLED_ADDRESS: u8 = 0x3C;

fn init_display(
    sda: Gpio21<'static>,
    scl: Gpio22<'static>,
    i2c_driver: I2C0<'static>,
) -> anyhow::Result<I2cDriver<'static>> {
    let config = I2cConfig::new()
        .baudrate(Hertz(100_000));

    let mut i2c = I2cDriver::new(
        i2c_driver,
        sda,
        scl,
        &config,
    )?;

    println!("Initializing OLED at 0x3C");

    let commands: &[&[u8]] = &[
        &[0x00, 0xAE],       // Display off
        &[0x00, 0xD5, 0x80], // Clock
        &[0x00, 0xA8, 0x3F], // Multiplex ratio: 64 rows
        &[0x00, 0xD3, 0x00], // Display offset
        &[0x00, 0x40],       // Start line
        &[0x00, 0x8D, 0x14], // Charge pump
        &[0x00, 0xA1],       // Segment remap
        &[0x00, 0xC8],       // COM scan direction
        &[0x00, 0xDA, 0x12], // COM pins
        &[0x00, 0x81, 0xCF], // Contrast
        &[0x00, 0xD9, 0xF1], // Pre-charge
        &[0x00, 0xDB, 0x40], // VCOMH
        &[0x00, 0xA4],       // Display follows RAM
        &[0x00, 0xA6],       // Normal display
        &[0x00, 0x20, 0x00], // Horizontal addressing
        &[0x00, 0xAF],       // Display on
    ];

    for command in commands {
        i2c.write(OLED_ADDRESS, command, BLOCK)?;
    }

    // Fill 1024 bytes: 128 columns × 8 pages.
    for _ in 0..64 {
        let data = [
            0x40,
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
        ];

        i2c.write(OLED_ADDRESS, &data, BLOCK)?;
    }

    println!("OLED initialized; display should be white.");

    Ok(i2c)
}

fn main()  {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take().unwrap();

let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;
let i2c_driver = peripherals.i2c0;


    let _display = init_display(sda, scl, i2c_driver);



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


    loop {
        std::thread::sleep(Duration::from_secs(1));
    }


    // Ok(())

}

fn delay(s:u64){
    std::thread::sleep(Duration::from_secs(s));
}

fn init_server(led_pin:Arc<Mutex<PinDriver<'static,Output>>>,servo1:Arc<Mutex<LedcDriver<'static>>>,servo2:Arc<Mutex<LedcDriver<'static>>>) -> anyhow::Result<EspHttpServer<'static>,anyhow::Error>{

    let mut server = EspHttpServer::new(&Default::default()).unwrap();

    let led_pin_root = Arc::clone(&led_pin);


    server.fn_handler("/", embedded_svc::http::Method::Get,move |req| -> anyhow::Result<()> {
    let mut response = req.into_ok_response()?;
        response.write("hello from esp32".as_bytes())?;
        led_pin_root.lock().unwrap().toggle();
        Ok(())
    })?;
    
    server.fn_handler("/led", embedded_svc::http::Method::Post,move |mut req| -> anyhow::Result<()> {

        let mut buffer = [0u8;1024];

        let len = req.read(&mut buffer)?;

        let received = std::str::from_utf8(&buffer[..len])?.trim();

        let number:u64 = received.parse()?;

        println!("number:{:?}",number);

        let mut response = req.into_ok_response()?;
        response.write("This is led route".as_bytes())?;
        
        for i in 0..number {
            println!("Iteration:{}",i);
            led_pin.lock().unwrap().set_high();
            delay(1);
            led_pin.lock().unwrap().set_low();
            delay(1);
        }

        Ok(())
    })?;



    fn interpolate(angle:u32,min:u32,max:u32)->u32{
angle * (max - min)/180 + min
    }

    fn rotate_servo(servo:Arc<Mutex<LedcDriver<'static>>>,angle:u32){
    let max_duty = servo.lock().unwrap().get_max_duty();

    let min = max_duty / 40;
    let max = max_duty / 8;

        servo.lock().unwrap().set_duty(interpolate(angle,min,max)).unwrap();
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
    move |mut req| -> anyhow::Result<()> {
            for i in 0..6 {
                rotate_servo(Arc::clone(&servo_for_loop), (i+1) * 30);
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
    move |mut req| -> anyhow::Result<()> {
            for i in 0..6 {
                rotate_servo(Arc::clone(&servo1_for_multi), (i+1) * 30);
                delay(1);
                rotate_servo(Arc::clone(&servo2_for_multi), (i+1) * 30);
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
    move |mut req| -> anyhow::Result<()> {
            for i in 0..6 {
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

fn scan_wifi(modem:Modem<'static>) -> anyhow::Result<(),anyhow::Error>{

    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = EspWifi::new(modem, sysloop,Some(nvs))?;
    match wifi.scan() {
        Ok(networks)=>{
            println!("Networks found:{:?}",networks);
        },
        Err(err)=>{
            log::error!("Wifi scan failed:{:?}",err);
        }
    }


    log::info!("scan complete");

    Ok(())

}

fn simple_connect_wifi(modem:Modem<'static>) -> anyhow::Result<EspWifi<'static>,anyhow::Error>{


    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = EspWifi::new(modem, sysloop,Some(nvs))?;

    let ssid:String<32> = String::try_from("Airtel_Airtelwifi").unwrap();
    let password:String<64> = String::try_from("Hrishik@12345").unwrap();

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid,
        auth_method:AuthMethod::WPAWPA2Personal,
        password,
        ..Default::default()
    }));


    wifi.start()?;

    wifi.connect()?;

    while !wifi.is_connected().unwrap() {
        let config = wifi.get_configuration().unwrap();
        println!("Waiting for station : {:?}",config);
    }

    println!("Wifi connected");

    Ok(wifi)

}

