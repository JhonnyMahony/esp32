use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::nvs::{EspNvsPartition, NvsDefault};
use esp_idf_svc::ping::EspPing;
use esp_idf_svc::timer::{EspTimerService, Task};
use esp_idf_svc::wifi::{
    AsyncWifi, AuthMethod, ClientConfiguration, Configuration, EspWifi, PmfConfiguration,
    ScanMethod,
};

use heapless;

const SSID: &str = "moto ThinkPhone25_6106";
const PASS: &str = "qwerty1238";

pub fn wifi(
    modem: impl Peripheral<P = Modem> + 'static,
    sysloop: EspSystemEventLoop,
    nvs: Option<EspNvsPartition<NvsDefault>>,
    timer_service: EspTimerService<Task>,
) -> anyhow::Result<AsyncWifi<EspWifi<'static>>> {
    use futures::executor::block_on;
    let mut wifi = AsyncWifi::wrap(
        EspWifi::new(modem, sysloop.clone(), nvs)?,
        sysloop,
        timer_service.clone(),
    )?;

    block_on(connect_wifi(&mut wifi))?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    println!("Wifi DHCP info: {:?}", ip_info);

    EspPing::default().ping(
        ip_info.subnet.gateway,
        &esp_idf_svc::ping::Configuration::default(),
    )?;
    Ok(wifi)
}

async fn connect_wifi(wifi: &mut AsyncWifi<EspWifi<'static>>) -> anyhow::Result<()> {
    let ssid: heapless::String<32> =
        heapless::String::try_from(SSID).map_err(|_| anyhow::anyhow!("SSID too long"))?;
    let password: heapless::String<64> =
        heapless::String::try_from(PASS).map_err(|_| anyhow::anyhow!("Password too long"))?;

    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid,
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password,
        channel: None,
        scan_method: ScanMethod::FastScan,
        pmf_cfg: PmfConfiguration::NotCapable,
    });

    wifi.set_configuration(&wifi_configuration)?;

    wifi.start().await?;
    log::info!("Wifi started");

    wifi.connect().await?;
    log::info!("Wifi connected");

    wifi.wait_netif_up().await?;
    log::info!("Wifi netif up");

    Ok(())
}
