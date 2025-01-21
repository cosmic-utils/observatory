lazy_static::lazy_static!(
    pub static ref CPU_STATIC: CpuStatic = CpuStatic::load();
);

#[derive(zbus::zvariant::Type, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CpuStatic {
    // The model name of the CPU
    pub model: String,
    pub physical_cores: usize,
    pub logical_cores: usize,
}

impl CpuStatic {
    fn load() -> Self {
        let cpuid = raw_cpuid::CpuId::new();
        Self {
            model: cpuid
                .get_processor_brand_string()
                .unwrap()
                .as_str()
                .to_string(),
            physical_cores: num_cpus::get_physical(),
            logical_cores: num_cpus::get(),
        }
    }
}

#[derive(zbus::zvariant::Type, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CpuDynamic {
    pub speed: u64,
    pub usage: f32,
}

impl CpuDynamic {
    pub fn load(system: &sysinfo::System) -> Self {
        Self {
            speed: system
                .cpus()
                .iter()
                .max_by(|x, y| x.frequency().cmp(&y.frequency()))
                .unwrap()
                .frequency(),
            usage: system.global_cpu_usage(),
        }
    }
}
