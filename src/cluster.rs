use crate::{
    responses,
};

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
    GeneralOnOff(C0006), 0x0006,
    GeneralLevelControl(C0008), 0x0008,
    LightingColorControl(C0300), 0x0300
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
            _ => {},
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
            _ => {},
        }
    }
}

#[derive(Debug, Clone)]
pub struct C0300 {
    pub color_temperature: u16,
}

impl ClusterTrait for C0300 {
    fn new() -> Self {
        Self { color_temperature: 0 }
    }
    fn update(&mut self, msg: &responses::ReadAttributeResponse) {
        match msg.attr_enum {
            7 => self.color_temperature = (msg.data[0] as u16) << 8 | (msg.data[1] as u16) & 0xff,
            _ => {},
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
