#[derive(Debug, PartialEq, Copy, Clone)]
pub struct KeydDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub usage_page: u16,
    pub usage_id: u16,
}

impl KeydDeviceInfo {
    pub const fn new(vendor_id: u16, product_id: u16, usage_page: u16, usage_id: u16) -> Self {
        Self {
            vendor_id,
            product_id,
            usage_page,
            usage_id,
        }
    }
}
