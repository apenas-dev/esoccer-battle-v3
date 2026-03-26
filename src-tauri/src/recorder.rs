use cpal::traits::{DeviceTrait as _, HostTrait as _};

#[derive(serde::Serialize)]
pub struct DeviceResult {
    pub name: String,
}

fn all_hosts() -> Vec<cpal::Host> {
    cpal::ALL_HOSTS
        .iter()
        .map(|id| cpal::host_from_id(*id))
        .filter_map(|h| h.ok())
        .collect()
}

pub fn list_microphone() -> Vec<DeviceResult> {
    all_hosts()
        .into_iter()
        .map(|host| host.input_devices())
        .filter_map(|d| d.ok())
        .flat_map(|d| d.collect::<Vec<_>>())
        .filter_map(|d| d.name().ok())
        .map(|name| DeviceResult { name })
        .collect()
}
