use anyhow::{anyhow, Result};
use esp_idf_svc::{
    hal::{
        gpio::Pins,
        ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver, LEDC},
        units::Hertz,
    },
    http::{server::EspHttpServer, Method},
};
use std::{
    str::from_utf8,
    sync::{Arc, Mutex},
};

fn interpolate(angle: u32, max: u32, min: u32) -> u32 {
    angle * (max - min) / 180 + min
}

pub fn servo_entry(server: &mut EspHttpServer, ledc: LEDC, pins: Pins) -> Result<()> {
    let servo_timer = ledc.timer0;
    let servo_time_driver = LedcTimerDriver::new(
        servo_timer,
        &TimerConfig::new()
            .frequency(Hertz::from(50))
            .resolution(esp_idf_svc::hal::ledc::Resolution::Bits14),
    )?;
    let driver = Arc::new(Mutex::new(LedcDriver::new(
        ledc.channel3,
        servo_time_driver,
        pins.gpio2,
    )?));
    let max_duty = driver.lock().unwrap().get_max_duty();
    let min = max_duty / 40; // Adjust based on servo specs
    let max = max_duty / 8; // Adjust based on servo specs

    server.fn_handler("/servo", Method::Post, move |mut req| {
        let mut buffer = [0_u8; 6];
        let bytes_read = req.read(&mut buffer)?;
        let angle_string =
            from_utf8(&buffer[0..bytes_read]).map_err(|_| anyhow!("Invalid UTF-8 input"))?;
        let angle: u32 = angle_string
            .parse()
            .map_err(|_| anyhow!("Invalid angle format"))?;
        if angle > 180 {
            return Err(anyhow!("Angle must be between 0 and 180"));
        }
        driver
            .lock()
            .map_err(|_| anyhow!("Failed to acquire driver lock"))?
            .set_duty(interpolate(angle, max, min))
            .map_err(|_| anyhow!("Failed to set duty cycle"))?;
        Ok(())
    })?;

    Ok(())
}
