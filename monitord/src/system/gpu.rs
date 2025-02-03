
#[derive(zbus::zvariant::Type, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GpuStatic {
    name: String,
}

impl GpuStatic {
    pub(crate) async fn load() -> Vec<Self> {
             
    }   
}
