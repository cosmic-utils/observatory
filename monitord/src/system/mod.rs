pub mod cpu;
pub use cpu::CpuDynamic;
pub use cpu::CpuStatic;

pub mod memory;
use disk::DiskStatic;
use memory::MemoryDynamic;
use memory::MemoryStatic;

pub mod disk;

pub mod process;
pub use process::Process;

#[derive(zbus::zvariant::Type, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SystemSnapshot {
    pub processes: Vec<Process>,
    pub cpu: (CpuStatic, CpuDynamic),
    pub mem: (MemoryStatic, MemoryDynamic),
    pub disk: Vec<(DiskStatic, u8)>,
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

// === SYSTEM SERVER FOR DBUS ===
#[allow(unused)]
pub(crate) struct SystemSnapshotServer {
    system: sysinfo::System,

    cpu_static: CpuStatic,
    mem_static: MemoryStatic,
    disk_static: Vec<DiskStatic>,
}

#[allow(unused)]
impl SystemSnapshotServer {
    pub(crate) async fn run() -> zbus::Result<()> {
        let (cpu_static, mem_static, disk_static) =
            tokio::join!(CpuStatic::load(), MemoryStatic::load(), DiskStatic::load());

        let server = Self {
            system: sysinfo::System::new_all(),
            cpu_static,
            mem_static,
            disk_static,
        };

        let connection = zbus::connection::Builder::session()?
            .name("io.github.CosmicUtils.Observatory")?
            .serve_at("/io/github/CosmicUtils/Observatory", server)?
            .build()
            .await?;
        tracing::info!("monitord dbus created");

        loop {
            let server: zbus::object_server::InterfaceRef<SystemSnapshotServer> = connection
                .object_server()
                .interface("/io/github/CosmicUtils/Observatory")
                .await?;

            let system = &mut server.get_mut().await;
            let snapshot = system.load();

            server.snapshot(snapshot.await?).await?;

            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        Ok(())
    }

    pub(crate) async fn load(&mut self) -> zbus::Result<SystemSnapshot> {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.system
            .refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        let (processes, cpu_dynamic_info, mem_dynamic_info) = tokio::join!(
            Process::load_all(&self.system),
            CpuDynamic::load(&self.system),
            MemoryDynamic::load(&self.system),
        );
        Ok(SystemSnapshot {
            processes,
            cpu: (self.cpu_static.clone(), cpu_dynamic_info),
            mem: (self.mem_static.clone(), mem_dynamic_info),
            disk: self
                .disk_static
                .iter()
                .cloned()
                .zip(vec![0u8].into_iter())
                .collect::<Vec<(DiskStatic, u8)>>(),
        })
    }
}

#[allow(unused)]
#[zbus::interface(name = "io.github.CosmicUtils.Observatory.SystemSnapshot")]
impl SystemSnapshotServer {
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
