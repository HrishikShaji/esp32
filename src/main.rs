use anyhow::{self, Error};
use embedded_svc::wifi::{AuthMethod,ClientConfiguration,Configuration};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::EspWifi;
use heapless::String;


fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello,final sample world!");

    let peripherals = Peripherals::take().unwrap();

    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = EspWifi::new(peripherals.modem, sysloop,Some(nvs))?;



    wifi.start()?;

    log::info!("wifi started and scanning");

    match wifi.scan() {
        Ok(networks)=>{
            println!("Networks found:{:?}",networks);
        },
        Err(err)=>{
            log::error!("Wifi scan failed:{:?}",err);
        }
    }


    log::info!("scan complete");

    // simple_connect_wifi()?;

    Ok(())

}


fn simple_connect_wifi() -> anyhow::Result<()>{

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

    Ok(())
}

