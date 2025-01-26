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

    fn kill_process(&self, pid: u32) -> zbus::Result<bool>;

    fn term_process(&self, pid: u32) -> zbus::Result<bool>;
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
pub(crate) struct SystemServer {
    system: sysinfo::System,
}

#[allow(unused)]
impl SystemServer {
    pub(crate) fn new() -> Self {
        Self {
            system: sysinfo::System::new_all(),
        }
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

    pub(crate) async fn kill_process(&self, pid: u32) -> zbus::fdo::Result<bool> {
        zbus::fdo::Result::Ok(
            self.system
                .process(sysinfo::Pid::from_u32(pid))
                .ok_or(zbus::Error::InvalidField)?
                .kill(),
        )
    }

    pub(crate) async fn term_process(&self, pid: u32) -> zbus::fdo::Result<bool> {
        zbus::fdo::Result::Ok(
            self.system
                .process(sysinfo::Pid::from_u32(pid))
                .ok_or(zbus::Error::InvalidField)?
                .kill_with(sysinfo::Signal::Term)
                .ok_or(zbus::Error::InvalidField)?,
        )
    }
}
