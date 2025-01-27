lazy_static::lazy_static!(
    pub static ref CPU_STATIC: CpuStatic = CpuStatic::load();
);

#[derive(zbus::zvariant::Type, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CpuCache {
    pub size: usize,
    pub level: u8,
    pub cache_type: String,
}

#[derive(zbus::zvariant::Type, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CpuStatic {
    // The model name of the CPU
    pub model: String,
    pub physical_cores: usize,
    pub logical_cores: usize,
    pub caches: Vec<CpuCache>,
}

impl CpuStatic {
    fn load_cache(cpuid: &raw_cpuid::CpuId<raw_cpuid::CpuIdReaderNative>) -> Vec<CpuCache> {
        if let Some(intel_cache) = cpuid.get_cache_parameters() {
            let mut caches = Vec::new();
            for cache in intel_cache {
                let size = cache.associativity()
                    * cache.physical_line_partitions()
                    * cache.coherency_line_size()
                    * cache.sets();
                caches.push(CpuCache {
                    size,
                    level: cache.level(),
                    cache_type: cache.cache_type().to_string(),
                })
            }
            caches
        } else {
            Vec::new()
        }
    }

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
            caches: Self::load_cache(&cpuid),
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
