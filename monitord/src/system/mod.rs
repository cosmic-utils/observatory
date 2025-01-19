pub mod cpu;

pub mod process;
pub use process::Process;

#[derive(zbus::zvariant::Type, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SystemSnapshot {
    pub processes: Vec<Process>,
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
    pub(crate) fn load(system: &mut sysinfo::System) -> Self {
        system.refresh_cpu_all();
        system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        SystemSnapshot {
            processes: Process::load_all(system),
        }
    }
}

// === SYSTEM SERVER FOR DBUS ===
pub(crate) struct SystemServer {}
impl SystemServer {
    pub(crate) fn new() -> Self {
        Self {}
    }
}
#[zbus::interface(name = "io.github.CosmicUtils.Observatory.SystemSnapshot")]
impl SystemServer {
    #[zbus(signal)]
    pub(crate) async fn snapshot(
        signal_emitter: &zbus::object_server::SignalEmitter<'_>,
        instance: SystemSnapshot,
    ) -> zbus::Result<()>;
}
