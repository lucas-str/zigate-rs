use crate::responses;
use std::stringify;

mod basic;
use basic::C0000;

macro_rules! make_cluster {
    ( $($cluster_name:ident ($cluster:ident), $id:expr ),+ ) => {
        #[derive(Debug, Clone)]
        pub enum Cluster {
            $( $cluster_name($cluster), )+
            Unk(u16),
        }

        impl Cluster {
            pub fn new(id: u16) -> Self {
                match id {
                    $( $id => Self::$cluster_name($cluster::new()), )+
                    _ => Self::Unk(id),
                }
            }
            pub fn id(&self) -> u16 {
                match self {
                    $( Self::$cluster_name(_) => $id, )+
                    Self::Unk(id) => *id,
                }
            }
            pub fn name(&self) -> Option<String> {
                match self {
                    $( Self::$cluster_name(_) => Some(stringify!($cluster_name).into()), )+
                    Self::Unk(_) => None,
                }
            }
            pub fn update(&mut self, msg: &responses::ReadAttributeResponse) {
                match self {
                    $( Self::$cluster_name(cluster) => cluster.update(msg), )+
                    Self::Unk(_) => {},
                }
            }
        }
    }
}

make_cluster!(
    Basic(C0000),
    0x0000,
    GeneralOnOff(C0006),
    0x0006,
    GeneralLevelControl(C0008),
    0x0008,
    LightingColorControl(C0300),
    0x0300
);

trait ClusterTrait {
    fn new() -> Self;
    fn update(&mut self, msg: &responses::ReadAttributeResponse);
}

#[derive(Debug, Clone)]
pub struct C0006 {
    pub onoff: bool,
}

impl ClusterTrait for C0006 {
    fn new() -> Self {
        Self { onoff: false }
    }
    fn update(&mut self, msg: &responses::ReadAttributeResponse) {
        match msg.attr_enum {
            0 => self.onoff = msg.data_as_bool().unwrap_or(false),
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct C0008 {
    pub current_level: u8,
}

impl ClusterTrait for C0008 {
    fn new() -> Self {
        Self { current_level: 0 }
    }
    fn update(&mut self, msg: &responses::ReadAttributeResponse) {
        match msg.attr_enum {
            0 => self.current_level = msg.data_as_u8().unwrap_or(0),
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub enum ColorMode {
    HueSat,
    XY,
    Temp,
}

#[derive(Debug, Clone, Copy)]
pub struct ColorCapabilities {
    pub hue_sat: bool,
    pub enhanced_hue: bool,
    pub color_loop: bool,
    pub xy: bool,
    pub temp: bool,
}

#[derive(Debug, Clone, Default)]
pub struct C0300 {
    pub current_hue: Option<u8>,
    pub current_saturation: Option<u8>,
    pub current_x: Option<u16>,
    pub current_y: Option<u16>,
    pub color_temperature: Option<u16>,
    pub color_mode: Option<ColorMode>,
    pub color_capabilities: Option<ColorCapabilities>,
    pub color_temp_min: Option<u16>,
    pub color_temp_max: Option<u16>,
}

impl ClusterTrait for C0300 {
    fn new() -> Self {
        Self::default()
    }
    fn update(&mut self, msg: &responses::ReadAttributeResponse) {
        match msg.attr_enum {
            0x0000 => {
                self.current_hue = msg.data_as_u8().ok();
            }
            0x0001 => {
                self.current_saturation = msg.data_as_u8().ok();
            }
            0x0003 => {
                self.current_x = msg.data_as_u16().ok();
            }
            0x0004 => {
                self.current_y = msg.data_as_u16().ok();
            }
            0x0007 => {
                self.color_temperature = msg.data_as_u16().ok();
            }
            0x0008 => match msg.data_as_u8() {
                Ok(0) => self.color_mode = Some(ColorMode::HueSat),
                Ok(1) => self.color_mode = Some(ColorMode::XY),
                Ok(2) => self.color_mode = Some(ColorMode::Temp),
                Ok(n) => {
                    error!("Invalid color mode {}", n);
                    self.color_mode = None;
                }
                _ => {
                    self.color_mode = None;
                }
            },
            0x400a => {
                let caps = match msg.data_as_u16() {
                    Ok(caps) => caps,
                    Err(_) => return,
                };
                let color_caps = ColorCapabilities {
                    hue_sat: (caps & 0x1) != 0,
                    enhanced_hue: (caps & 0x2) != 0,
                    color_loop: (caps & 0x4) != 0,
                    xy: (caps & 0x8) != 0,
                    temp: (caps & 0x10) != 0,
                };
                self.color_capabilities = Some(color_caps);
            }
            0x400b => {
                self.color_temp_min = msg.data_as_u16().ok();
            }
            0x400c => {
                self.color_temp_max = msg.data_as_u16().ok();
            }
            _ => {}
        }
    }
}

//#[derive(Debug, Clone)]
//pub struct Cluster {
//    id: u16,
//}
//
//impl Cluster {
//    pub fn new(id: u16) -> Self {
//        Cluster { id }
//    }
//}
