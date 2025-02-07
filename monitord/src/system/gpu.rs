pub(crate) mod nvidia;

#[derive(zbus::zvariant::Type, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GpuStatic {
    pub name: String,
    pub driver: String,
    pub video_memory: u64,
}

#[derive(zbus::zvariant::Type, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GpuDynamic {
    pub usage: f32,
    pub enc: f32,
    pub dec: f32,
    pub video_mem: u64,
    pub procs: Vec<(u32, f32)>,
}

pub(crate) trait GpuBackend {
    fn get_static(&self) -> Result<Vec<GpuStatic>, Box<dyn std::error::Error>>;
    fn get_dynamic(&self) -> Result<Vec<GpuDynamic>, Box<dyn std::error::Error>>;
}
