pub(crate) struct Amd {
    gpu_handles: Vec<amdgpu_sysfs::gpu_handle::GpuHandle>,
}

impl Amd {
    pub(crate) fn init() -> Result<Self, Box<dyn std::error::Error>> {
        // Support up to 5 AMD GPUs
        let mut gpu_handles = Vec::new();
        for i in 0..5 {
            let path = std::path::PathBuf::from(format!("/sys/class/drm/card{i}/device"));
            tracing::info!("Attempting to read AMD GPU at {:?}", path.clone());

            match amdgpu_sysfs::gpu_handle::GpuHandle::new_from_path(path.clone()) {
                Ok(gpu_handle) => {
                    tracing::info!("Read GPU handle {:?}", path);
                    gpu_handles.push(gpu_handle);
                }
                Err(e) => {
                    if e.is_not_found() {
                        break;
                    }
                }
            }
        }

        Ok(Self { gpu_handles })
    }
}

impl super::GpuBackend for Amd {
    fn get_static(&self) -> Result<Vec<super::GpuStatic>, Box<dyn std::error::Error>> {
        self.gpu_handles
            .iter()
            .map(|gpu_handle| {
                Ok(super::GpuStatic {
                    name: gpu_handle
                        .get_pci_id()
                        .map(|(id0, id1)| {
                            tracing::info!("Parsing VID: {} and PID: {}", id0, id1);
                            let id0 = u16::from_str_radix(id0, 16);
                            let id1 = u16::from_str_radix(id1, 16);
                            tracing::info!("Parsed VID: {id0:?}, PID: {id1:?}");
                            let device = pci_ids::Device::from_vid_pid(
                                id0.unwrap_or_default(),
                                id1.unwrap_or_default(),
                            );
                            tracing::info!("Was a PCI Device: {}", device.is_some());
                            device
                                .map(|device| device.name().to_string())
                                .unwrap_or_default()
                        })
                        .unwrap_or_default(),
                    driver: gpu_handle.get_driver().to_string(),
                    video_memory: gpu_handle.get_total_vram()?,
                })
            })
            .collect()
    }

    fn get_dynamic(&self) -> Result<Vec<super::GpuDynamic>, Box<dyn std::error::Error>> {
        self.gpu_handles
            .iter()
            .map(|gpu_handle| {
                Ok(super::GpuDynamic {
                    usage: gpu_handle.get_busy_percent()? as f32,
                    enc: -1.0,
                    dec: -1.0,
                    video_mem: gpu_handle.get_used_vram()?,
                    procs: vec![],
                })
            })
            .collect()
    }
}
