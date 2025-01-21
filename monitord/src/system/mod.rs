pub mod cpu;
use cpu::CpuDynamic;
pub use cpu::CpuStatic;

pub mod process;
pub use process::Process;

#[derive(zbus::zvariant::Type, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SystemSnapshot {
    pub processes: Vec<Process>,
    pub cpu_static_info: CpuStatic,
    pub cpu_dynamic_info: CpuDynamic,
}

#[zbus::proxy(
    interface = "io.github.CosmicUtils.Observatory.SystemSnapshot",
    default_service = "io.github.CosmicUtils.Observatory",
    default_path = "/io/github/CosmicUtils/Observatory"
)]
pub trait SystemSnapshot {
    #[zbus(signal)]
    fn snapshot(&self, instance: SystemSnapshot) -> zbus::Result<()>;
}

impl SystemSnapshot {
    #[allow(unused)]
    pub(crate) fn load(system: &mut sysinfo::System) -> Self {
        system.refresh_cpu_all();
        system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        SystemSnapshot {
            processes: Process::load_all(system),
            cpu_static_info: cpu::CPU_STATIC.clone(),
            cpu_dynamic_info: CpuDynamic::load(system),
        }
    }
}

// === SYSTEM SERVER FOR DBUS ===
#[allow(unused)]
pub(crate) struct SystemServer {}

#[allow(unused)]
impl SystemServer {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

#[allow(unused)]
#[zbus::interface(name = "io.github.CosmicUtils.Observatory.SystemSnapshot")]
impl SystemServer {
    #[zbus(signal)]
    pub(crate) async fn snapshot(
        signal_emitter: &zbus::object_server::SignalEmitter<'_>,
        instance: SystemSnapshot,
    ) -> zbus::Result<()>;
}
