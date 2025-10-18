use std::{
    str::from_utf8,
    sync::{Arc, Mutex},
};

use esp_idf_svc::{
    hal::{
        gpio::{PinDriver, Pins},
        ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver, LEDC},
        units::Hertz,
    },
    http::{server::EspHttpServer, Method},
    io::Read,
};

pub struct Color {
    r: LedcDriver<'static>,
    g: LedcDriver<'static>,
    b: LedcDriver<'static>,
}

impl Color {
    fn new(r: LedcDriver<'static>, g: LedcDriver<'static>, b: LedcDriver<'static>) -> Self {
        Self { r, g, b }
    }
    fn set_color_str(&mut self, color: &str) -> anyhow::Result<()> {
        if color.len() != 6 {
            return Err(anyhow::anyhow!("Color string must be 6 characters long"));
        }
        let r = u8::from_str_radix(&color[0..2], 16)
            .map_err(|_| anyhow::anyhow!("Invalid red value"))? as u32;
        let g = u8::from_str_radix(&color[2..4], 16)
            .map_err(|_| anyhow::anyhow!("Invalid green value"))? as u32;
        let b = u8::from_str_radix(&color[4..6], 16)
            .map_err(|_| anyhow::anyhow!("Invalid blue value"))? as u32;
        self.r.set_duty(r)?;
        self.g.set_duty(g)?;
        self.b.set_duty(b)?;
        Ok(())
    }
}

pub fn led_entry(server: &mut EspHttpServer, ledc: LEDC, pins: Pins) -> anyhow::Result<()> {
    let led_timer = ledc.timer0;
    let led_time_driver =
        LedcTimerDriver::new(led_timer, &TimerConfig::new().frequency(Hertz::from(1000))).unwrap();
    let red_led = LedcDriver::new(ledc.channel1, &led_time_driver, pins.gpio6).unwrap();
    let green_led = LedcDriver::new(ledc.channel2, &led_time_driver, pins.gpio7).unwrap();
    let blue_led = LedcDriver::new(ledc.channel3, &led_time_driver, pins.gpio8).unwrap();
    let color_ctrl = Arc::new(Mutex::new(Color::new(red_led, green_led, blue_led)));
    toggle_color(server, color_ctrl).unwrap();
    Ok(())
}

pub fn toggle_led(server: &mut EspHttpServer, pins: Pins) -> anyhow::Result<()> {
    let led_pin = Arc::new(Mutex::new(PinDriver::output(pins.gpio1).unwrap()));
    server.fn_handler("/toggle_led", Method::Get, move |req| {
        let mut response = req.into_ok_response()?; // Create OK response
        response.write("Hello from Esp32".as_bytes())?; // Write response body
        led_pin.lock().unwrap().toggle()?; // Toggle LED
        Ok::<(), anyhow::Error>(()) // Return success
    })?;
    Ok(())
}

pub fn toggle_color(
    server: &mut EspHttpServer,
    ledc_driver: Arc<Mutex<Color>>,
) -> anyhow::Result<()> {
    server.fn_handler("/color", Method::Post, move |mut req| {
        let mut buffer = [0_u8; 6];
        req.read_exact(&mut buffer).unwrap();
        let color = from_utf8(&buffer).unwrap();
        let response_body = format!("Toggling LED with color: {}", color);
        let mut response = req.into_ok_response()?; // Create OK response
        ledc_driver.lock().unwrap().set_color_str(color)?;
        response.write(response_body.as_bytes())?; // Write response body
        Ok::<(), anyhow::Error>(()) // Return success
    })?;
    Ok(())
}
