#[derive(zbus::zvariant::Type, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct MemoryStatic {
    pub resident_capacity: usize,
    pub swap_capacity: usize,
    pub speed: usize,
}

impl MemoryStatic {
    pub(crate) async fn load() -> Self {
        let sys = sysinfo::System::new_with_specifics(
            sysinfo::RefreshKind::nothing().with_memory(sysinfo::MemoryRefreshKind::everything()),
        );

        Self {
            resident_capacity: sys.total_memory() as usize,
            swap_capacity: sys.total_swap() as usize,
            speed: 0, // TODO: Find a way to get this
        }
    }
}

#[derive(zbus::zvariant::Type, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct MemoryDynamic {
    pub resident: usize,
    pub swap: usize,
}

impl MemoryDynamic {
    pub(crate) async fn load(system: &sysinfo::System) -> Self {
        Self {
            resident: system.used_memory() as usize,
            swap: system.used_swap() as usize,
        }
    }
}
