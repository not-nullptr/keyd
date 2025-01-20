mod clipboard;
mod device;
mod info;
mod scan;

use std::{
    cell::RefCell,
    error::Error,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use device::KeydDeviceInfo;
use env_logger::Env;
use hidapi::HidApi;
use log::{error, info};
use scan::Scanner;

// const VENDOR_ID: u16 = 0xFC32;
// const PRODUCT_ID: u16 = 0x0287;
// const USAGE_PAGE: u16 = 0xFF60;
// const USAGE: u16 = 0x61;
// const REPORT_LENGTH: usize = 32;

const DISCOVERABLE_DEVICES: [KeydDeviceInfo; 1] = [KeydDeviceInfo {
    vendor_id: 0xFC32,
    product_id: 0x0287,
    usage_page: 0xFF60,
    usage_id: 0x61,
}];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    let api = HidApi::new()?;
    let devices = DISCOVERABLE_DEVICES.to_vec();
    let scanner = Scanner::new(api, devices);
    scanner.scan_devices().await;
    Ok(())
}
