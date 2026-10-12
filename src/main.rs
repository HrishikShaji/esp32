use std::time::Duration;

use anyhow::{self, Error};
use embedded_svc::wifi::{AuthMethod,ClientConfiguration,Configuration};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::http::server::EspHttpServer;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AsyncWifi, EspWifi};
use heapless::String;


fn main()  {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello,final sample world!");


    let _wifi = simple_connect_wifi();
    let _server = init_server();



    loop {
        std::thread::sleep(Duration::from_secs(1));
    }


    // Ok(())

}

fn init_server() -> anyhow::Result<EspHttpServer<'static>,anyhow::Error>{

    let mut server = EspHttpServer::new(&Default::default()).unwrap();

    server.fn_handler("/", embedded_svc::http::Method::Get, |req| -> anyhow::Result<()> {
    let mut response = req.into_ok_response()?;
        response.write("hello from esp32".as_bytes())?;
        Ok(())
    })?;

    Ok(server)
}

fn scan_wifi() -> anyhow::Result<(),anyhow::Error>{
    let peripherals = Peripherals::take().unwrap();

    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = EspWifi::new(peripherals.modem, sysloop,Some(nvs))?;
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

fn simple_connect_wifi() -> anyhow::Result<EspWifi<'static>,anyhow::Error>{

    let peripherals = Peripherals::take().unwrap();

    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = EspWifi::new(peripherals.modem, sysloop,Some(nvs))?;

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

