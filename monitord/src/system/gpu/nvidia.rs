pub(crate) struct Nvidia {
    nvml: nvml_wrapper::Nvml,
}

impl Nvidia {
    pub(crate) fn init() -> Result<Self, Box<dyn std::error::Error>> {
        let nvml = nvml_wrapper::Nvml::init()?;
        Ok(Self { nvml })
    }
}

impl super::GpuBackend for Nvidia {
    fn get_static(&self) -> Result<Vec<super::GpuStatic>, Box<dyn std::error::Error>> {
        let device_count = self.nvml.device_count().unwrap_or_default();
        let mut devices = Vec::new();
        for i in 0..device_count {
            devices.push(self.nvml.device_by_index(i)?);
        }

        devices
            .iter()
            .map(|device| {
                Ok(super::GpuStatic {
                    name: device.name().unwrap_or_default(),
                    driver: self.nvml.sys_driver_version().unwrap_or_default(),
                    video_memory: device
                        .memory_info()
                        .map(|meminfo| meminfo.total)
                        .unwrap_or_default(),
                })
            })
            .collect()
    }

    fn get_dynamic(&self) -> Result<Vec<super::GpuDynamic>, Box<dyn std::error::Error>> {
        let device_count = self.nvml.device_count().unwrap_or_default();

        let mut devices = Vec::new();
        for i in 0..device_count {
            devices.push(self.nvml.device_by_index(i)?);
        }

        devices
            .iter()
            .map(|device| {
                Ok(super::GpuDynamic {
                    usage: device
                        .utilization_rates()
                        .map(|util| util.gpu)
                        .unwrap_or_default() as f32,
                    enc: device
                        .encoder_utilization()
                        .map(|util| util.utilization)
                        .unwrap_or_default() as f32,
                    dec: device
                        .decoder_utilization()
                        .map(|util| util.utilization)
                        .unwrap_or_default() as f32,
                    video_mem: device
                        .memory_info()
                        .map(|meminfo| meminfo.used)
                        .unwrap_or_default(),
                    procs: device
                        .process_utilization_stats(None)
                        .map(|process_list| {
                            process_list
                                .iter()
                                .map(|proc_sample| (proc_sample.pid, proc_sample.sm_util as f32))
                                .collect::<Vec<(u32, f32)>>()
                        })
                        .unwrap_or_default(),
                })
            })
            .collect()
    }
}
