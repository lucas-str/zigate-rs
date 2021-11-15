use crate::cluster::ClusterTrait;
use crate::responses;

#[derive(Debug, Clone, Default)]
pub struct C0000 {
    pub zcl_version: u8,
    pub application_version: u8,
    pub stack_version: u8,
    pub hw_version: u8,
    pub manufacturer_name: String,
    pub model_identifier: String,
    pub date_code: String,
    pub power_source: u8,
    pub location_description: String,
    pub physical_environment: u8,
    pub device_enabled: bool,
    pub alarm_mask: u8,
    pub disable_local_config: u8,
    pub sw_build_id: String,
}

impl ClusterTrait for C0000 {
    fn new() -> Self {
        Self::default()
    }
    fn update(&mut self, msg: &responses::ReadAttributeResponse) {
        match msg.attr_enum {
            0x0 => self.zcl_version = msg.data_as_u8().unwrap_or(0),
            0x1 => self.application_version = msg.data_as_u8().unwrap_or(0),
            0x2 => self.stack_version = msg.data_as_u8().unwrap_or(0),
            0x3 => self.hw_version = msg.data_as_u8().unwrap_or(0),
            0x4 => self.manufacturer_name = msg.data_as_str().unwrap_or("").into(),
            0x5 => self.model_identifier = msg.data_as_str().unwrap_or("").into(),
            0x6 => self.date_code = msg.data_as_str().unwrap_or("").into(),
            0x7 => self.power_source = msg.data_as_u8().unwrap_or(0), // FIXME make enum
            0x10 => self.location_description = msg.data_as_str().unwrap_or("").into(),
            0x11 => self.physical_environment = msg.data_as_u8().unwrap_or(0), // FIXME make enum
            0x12 => self.device_enabled = msg.data_as_bool().unwrap_or(true),
            0x13 => self.alarm_mask = msg.data_as_u8().unwrap_or(0), // FIXME make enum ?
            0x14 => self.disable_local_config = msg.data_as_u8().unwrap_or(0), // FIXME make enum ?
            0x16 => self.sw_build_id = msg.data_as_str().unwrap_or("").into(),
            _ => {}
        }
    }
}
