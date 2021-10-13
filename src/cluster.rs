use crate::responses;
use std::stringify;

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
            0 => self.onoff = msg.data[0] != 0,
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
            0 => self.current_level = msg.data[0],
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

#[derive(Debug, Clone)]
pub struct C0300 {
    pub current_hue: Option<u8>,
    pub current_saturation: Option<u8>,
    pub current_x: Option<u16>,
    pub current_y: Option<u16>,
    pub color_temperature: Option<u16>,
    pub color_mode: Option<ColorMode>,
    pub color_capabilities: Option<ColorCapabilities>,
}

impl ClusterTrait for C0300 {
    fn new() -> Self {
        Self {
            current_hue: None,
            current_saturation: None,
            current_x: None,
            current_y: None,
            color_temperature: None,
            color_mode: None,
            color_capabilities: None,
        }
    }
    fn update(&mut self, msg: &responses::ReadAttributeResponse) {
        match msg.attr_enum {
            0x0000 => {
                self.current_hue = Some(msg.data[0]);
            }
            0x0001 => {
                self.current_saturation = Some(msg.data[0]);
            }
            0x0003 => {
                let cur_x = (msg.data[0] as u16) << 8 | (msg.data[1] as u16) & 0xff;
                self.current_x = Some(cur_x);
            }
            0x0004 => {
                let cur_y = (msg.data[0] as u16) << 8 | (msg.data[1] as u16) & 0xff;
                self.current_y = Some(cur_y);
            }
            0x0007 => {
                let temp = (msg.data[0] as u16) << 8 | (msg.data[1] as u16) & 0xff;
                self.color_temperature = Some(temp);
            }
            0x0008 => match msg.data[0] {
                0 => self.color_mode = Some(ColorMode::HueSat),
                1 => self.color_mode = Some(ColorMode::XY),
                2 => self.color_mode = Some(ColorMode::Temp),
                inv => {
                    error!("Invalid color mode {}", inv);
                    self.color_mode = None;
                }
            },
            0x400a => {
                let caps = (msg.data[0] as u16) << 8 | (msg.data[1] as u16) & 0xff;
                let color_caps = ColorCapabilities {
                    hue_sat: (caps & 0x1) != 0,
                    enhanced_hue: (caps & 0x2) != 0,
                    color_loop: (caps & 0x4) != 0,
                    xy: (caps & 0x8) != 0,
                    temp: (caps & 0x10) != 0,
                };
                self.color_capabilities = Some(color_caps);
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
