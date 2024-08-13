
use std::{sync::Arc, time::Duration};

use buttplug::{
    client::{ButtplugClient, ButtplugClientDevice, ButtplugClientError, ButtplugClientEvent, ScalarValueCommand},
    core::{
        connector::new_json_ws_client_connector, message::ClientGenericDeviceMessageAttributes,
    },
};
use futures::stream::StreamExt;
use tokio::sync::Mutex;

pub async fn start_buttplug() -> Result<ButtplugClient, ButtplugClientError> {
    let connector = new_json_ws_client_connector("ws://127.0.0.1:12345/buttplug");
    let client = ButtplugClient::new("Swarm Bliss Client");
    client.connect(connector).await?;
    let mut events = client.event_stream();

    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            match event {
                ButtplugClientEvent::DeviceAdded(device) => {
                    log::info!("Device {} Connected!", device.name());
                }
                ButtplugClientEvent::DeviceRemoved(info) => {
                    log::info!("Device {} Removed!", info.name());
                }
                ButtplugClientEvent::ScanningFinished => {
                    log::info!("Device scanning is finished!");
                }
                _ => {}
            }
        }
    });

    log::info!("Device successfully connected!");

    client.start_scanning().await?;
    log::info!("Scanning for devices. Press enter when ready.");
    Ok(client)

}

pub async fn display_device(client: Arc<Mutex<ButtplugClient>>) -> Result<Arc<ButtplugClientDevice>, ButtplugClientError> {
    let locked_client = client.lock().await;
    locked_client.stop_scanning().await?;

    log::info!("Client currently knows about these devices:");
    for device in locked_client.devices() {
        fn print_attrs(attrs: &Vec<ClientGenericDeviceMessageAttributes>) {
            for attr in attrs {
                log::info!(
                    "{}: {} - Steps: {}",
                    attr.actuator_type(),
                    attr.feature_descriptor(),
                    attr.step_count()
                );
            }
        }
        log::info!("{} supports these actions:", device.name());
        if let Some(attrs) = device.message_attributes().scalar_cmd() {
            print_attrs(attrs);
        }
        print_attrs(&device.rotate_attributes());
        print_attrs(&device.linear_attributes());
        log::info!("Battery: {}", device.has_battery_level());
        log::info!("RSSI: {}", device.has_rssi_level());
    }

    return Ok(locked_client.devices()[0].to_owned());
}

pub async fn vibrate(device: &ButtplugClientDevice, speed: &ScalarValueCommand, delay: Duration) -> Result<(), ButtplugClientError> {
    device.vibrate(speed).await?;
    tokio::time::sleep(delay).await;
    device.stop().await?;
    Ok(())
}
