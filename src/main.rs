use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use esp_idf_svc::hal::units::Hertz;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        gpio::PinDriver,
        ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver},
        prelude::Peripherals,
    },
    http::{self, server::EspHttpServer},
    nvs::EspDefaultNvsPartition,
    timer::EspTaskTimerService,
};

use crate::wifi::wifi;

//mod led;
mod servo;
mod wifi;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("ESP initialized");

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take().unwrap();
    let timer_service = EspTaskTimerService::new().unwrap();
    let nvs = EspDefaultNvsPartition::take().ok();
    let _wifi = wifi(peripherals.modem, sysloop, nvs, timer_service);

    let mut server = EspHttpServer::new(&Default::default()).unwrap();
    servo::servo_entry(&mut server, peripherals.ledc, peripherals.pins).unwrap();
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
