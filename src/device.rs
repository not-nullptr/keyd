#[derive(Debug, PartialEq, Copy, Clone)]
pub struct KeydDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub usage_page: u16,
    pub usage_id: u16,
}
