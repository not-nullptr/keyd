use bincode::{config, Decode, Encode};
use hidapi::HidApi;
use log::{error, info, warn};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{sync::RwLock, task};

// const REPORT_LENGTH: usize = 32;

use crate::{device::KeydDeviceInfo, info::InfoMonitor};

#[derive(Debug, Clone, Encode, Decode)]
pub struct ToKeyboard {
    pub time: u32,
}

pub struct Scanner {
    api: Rc<RefCell<HidApi>>,
    device_list: Vec<KeydDeviceInfo>,
    active_threads: Arc<RwLock<Vec<KeydDeviceInfo>>>,
    info_monitor: Arc<RwLock<InfoMonitor>>,
}

impl Scanner {
    pub fn new(api: HidApi, device_list: Vec<KeydDeviceInfo>) -> Self {
        let mut monitor = InfoMonitor::new();
        monitor.begin_montioring();
        Self {
            api: Rc::new(RefCell::new(api)),
            device_list,
            active_threads: Arc::new(RwLock::new(Vec::new())),
            info_monitor: Arc::new(RwLock::new(monitor)),
        }
    }

    pub async fn scan_devices(&self) {
        info!("scanning for your keeb");
        loop {
            if let Err(e) = self.api.borrow_mut().refresh_devices() {
                error!("failed to refresh devices: {}", e);
                continue;
            };
            let api = self.api.borrow();
            let devices = api
                .device_list()
                .filter(|d| {
                    self.device_list.iter().any(|device| {
                        d.vendor_id() == device.vendor_id
                            && d.product_id() == device.product_id
                            && d.usage_page() == device.usage_page
                            && d.usage() == device.usage_id
                    })
                })
                .collect::<Vec<_>>();

            for device in devices {
                let device_info = self
                    .device_list
                    .iter()
                    .find(|d| {
                        device.vendor_id() == d.vendor_id
                            && device.product_id() == d.product_id
                            && device.usage_page() == d.usage_page
                            && device.usage() == d.usage_id
                    })
                    .unwrap();

                let device_clone = match device.open_device(&api) {
                    Ok(d) => d,
                    Err(e) => {
                        error!("failed to open device: {}", e);
                        continue;
                    }
                };
                let device_arc = Arc::new(Mutex::new(device_clone));
                let device_info_clone = device_info.clone();

                let mut threads_lock = self.active_threads.write().await;
                if threads_lock.iter().any(|d| *d == *device_info) {
                    continue;
                }

                threads_lock.push(device_info_clone.clone());

                let threads_clone = Arc::clone(&self.active_threads);
                let info_monitor_clone = Arc::clone(&self.info_monitor);

                let device_arc_clone = Arc::clone(&device_arc);
                task::spawn(async move {
                    Self::handle_device(device_arc_clone, info_monitor_clone).await;
                    warn!("device thread exited! removing from active threads rn");
                    let mut threads_lock = threads_clone.write().await;
                    threads_lock.retain(|d| *d != device_info_clone);
                });
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    async fn handle_device(
        device: Arc<Mutex<hidapi::HidDevice>>,
        info_monitor: Arc<RwLock<InfoMonitor>>,
    ) {
        loop {
            let name = device
                .lock()
                .unwrap()
                .get_product_string()
                .unwrap_or_default()
                .unwrap_or("unknown".to_string())
                .to_lowercase();

            let request_data =
                Self::get_request_data(Arc::clone(&device), Arc::clone(&info_monitor)).await;

            let send_result = device.lock().unwrap().write(&request_data);

            let Ok(size) = send_result else {
                warn!("failed to send data, exiting thread!");
                break;
            };

            info!("{}: sent {} bytes!", name, size);

            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    async fn get_request_data(
        _device: Arc<Mutex<hidapi::HidDevice>>,
        info_monitor: Arc<RwLock<InfoMonitor>>,
    ) -> Vec<u8> {
        let mon = info_monitor.read().await;
        let time = *mon.time.read().await;
        let level = *mon.level.read().await;

        // get u32 in bytes
        let time_bytes = time.to_le_bytes();
        let level_bytes = level.to_le_bytes();
        let mut request_data = vec![0x00, 0xFF, 0xCC, 0x00, 0xAA];
        request_data.extend_from_slice(&time_bytes);
        request_data.extend_from_slice(&level_bytes);
        // pad to 16 bytes
        while request_data.len() < 16 {
            request_data.push(0x00);
        }
        request_data
    }
}
