use crate::{
    responses,
    cluster::Cluster,
};

#[derive(Debug, Clone)]
pub struct Device {
    pub id: Option<u8>,
    pub short_address: u16,
    pub ieee_address: u64,
    pub power_source: Option<bool>,
    pub link_quality: Option<u8>,
    pub endpoints: Vec<Endpoint>,
}

impl Device {
    pub fn new(short_address: u16, ieee_address: u64) -> Self {
        Self {
            short_address,
            ieee_address,
            id: None,
            power_source: None,
            link_quality: None,
            endpoints: Vec::new(),
        }
    }

    pub fn from_devices_list_elem(device: responses::Device) -> Self {
        Self {
            short_address: device.short_address,
            ieee_address: device.ieee_address,
            id: Some(device.id),
            power_source: Some(device.power_source),
            link_quality: Some(device.link_quality),
            endpoints: Vec::new(),
        }
    }

    pub fn from_device_announce(msg: &responses::DeviceAnnounce) -> Self {
        Self {
            id: None,
            short_address: msg.short_address,
            ieee_address: msg.ieee_address,
            power_source: None,
            link_quality: None,
            endpoints: Vec::new(),
        }
    }

    pub fn add_endpoints(&mut self, endpoints: &Vec<u8>) {
        for endpoint in endpoints {
            self.endpoints.push(Endpoint::new(*endpoint));
        }
    }

    pub fn set_endpoints_clusters(&mut self, msg: &responses::SimpleDescriptorResponse) {
        for endpoint in self.endpoints.as_mut_slice() {
            let mut in_clusters = Vec::new();
            let mut out_clusters = Vec::new();
            for id in &msg.in_cluster_list {
                in_clusters.push(Cluster::new(*id));
            }
            for id in &msg.out_cluster_list {
                out_clusters.push(Cluster::new(*id));
            }
            endpoint.set_in_clusters(in_clusters);
            endpoint.set_out_clusters(out_clusters);
        }
    }

    pub fn get_endpoint(&self, id: u8) -> Option<&Endpoint> {
        for endpoint in &self.endpoints {
            if endpoint.id == id {
                return Some(endpoint)
            }
        }
        None
    }

    fn get_mut_endpoint(&mut self, id: u8) -> Option<&mut Endpoint> {
        for endpoint in &mut self.endpoints {
            if endpoint.id == id {
                return Some(endpoint)
            }
        }
        None
    }

    pub fn update_cluster(&mut self, msg: &responses::ReadAttributeResponse) {
        if let Some(endpoint) = self.get_mut_endpoint(msg.endpoint) {
            endpoint.update_cluster(&msg);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Endpoint {
    pub id: u8,
    in_clusters: Vec<Cluster>,
    out_clusters: Vec<Cluster>,
}

impl Endpoint {
    pub fn new(id: u8) -> Self {
        Self {
            id,
            in_clusters: Vec::new(),
            out_clusters: Vec::new(),
        }
    }

    pub fn set_in_clusters(&mut self, clusters: Vec<Cluster>) {
        self.in_clusters = clusters;
    }

    pub fn set_out_clusters(&mut self, clusters: Vec<Cluster>) {
        self.out_clusters = clusters;
    }

    pub fn get_in_clusters(&self) -> &Vec<Cluster> {
        &self.in_clusters
    }

    fn get_mut_in_cluster(&mut self, id: u16) -> Option<&mut Cluster> {
        for cluster in &mut self.in_clusters {
            if cluster.id() == id {
                return Some(cluster)
            }
        }
        None
    }

    pub fn update_cluster(&mut self, msg: &responses::ReadAttributeResponse) {
        if let Some(cluster) = self.get_mut_in_cluster(msg.cluster_id) {
            cluster.update(&msg);
        }
    }
}
