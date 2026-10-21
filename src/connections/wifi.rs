use anyhow::{self};
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_hal::modem::Modem;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::EspWifi;
use heapless::String;

pub fn scan_wifi(modem: Modem<'static>) -> anyhow::Result<(), anyhow::Error> {
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = EspWifi::new(modem, sysloop, Some(nvs))?;
    match wifi.scan() {
        Ok(networks) => {
            println!("Networks found:{:?}", networks);
        }
        Err(err) => {
            log::error!("Wifi scan failed:{:?}", err);
        }
    }

    log::info!("scan complete");

    Ok(())
}

pub fn simple_connect_wifi(
    modem: Modem<'static>,
) -> anyhow::Result<EspWifi<'static>, anyhow::Error> {
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = EspWifi::new(modem, sysloop, Some(nvs))?;

    let ssid: String<32> = String::try_from("Airtel_Airtelwifi").unwrap();
    let password: String<64> = String::try_from("Hrishik@12345").unwrap();

    let _wifi = wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid,
        auth_method: AuthMethod::WPAWPA2Personal,
        password,
        ..Default::default()
    }));

    wifi.start()?;

    wifi.connect()?;

    while !wifi.is_connected().unwrap() {
        let config = wifi.get_configuration().unwrap();
        println!("Waiting for station : {:?}", config);
    }

    println!("Wifi connected");

    Ok(wifi)
}
