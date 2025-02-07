pub mod cpu;
use std::sync::Arc;

pub use cpu::CpuDynamic;
pub use cpu::CpuStatic;

pub mod memory;
use memory::MemoryDynamic;
use memory::MemoryStatic;

pub mod disk;
use disk::DiskDynamic;
use disk::DiskStatic;

pub mod gpu;
use gpu::GpuDynamic;
use gpu::GpuStatic;

pub mod process;
pub use process::Process;

#[derive(zbus::zvariant::Type, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SystemSnapshot {
    pub processes: Vec<Process>,
    pub cpu: (CpuStatic, CpuDynamic),
    pub mem: (MemoryStatic, MemoryDynamic),
    pub disks: Vec<(DiskStatic, DiskDynamic)>,
    pub gpus: Vec<(GpuStatic, GpuDynamic)>,
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
    disks: sysinfo::Disks,
    cpu_static: CpuStatic,
    mem_static: MemoryStatic,
    disk_static: Vec<DiskStatic>,
    gpu_static: Vec<GpuStatic>,
}

#[allow(unused)]
impl SystemSnapshotServer {
    pub(crate) async fn run() -> zbus::Result<()> {
        let (cpu_static, mem_static, disk_static, backends) = tokio::join!(
            CpuStatic::load(),
            MemoryStatic::load(),
            DiskStatic::load(),
            async {
                let mut backends = Vec::new();
                match gpu::nvidia::Nvidia::init() {
                    Ok(nvidia) => backends.push(Arc::new(nvidia) as Arc<dyn gpu::GpuBackend>),
                    Err(e) => tracing::error!("Error loading NVML: {}", e.to_string()),
                }
                backends
            }
        );

        let gpu_static = tokio::join!(async {
            let mut gpu_static = Vec::new();
            for backend in backends.iter() {
                match backend.get_static() {
                    Ok(static_inf) => gpu_static.extend(static_inf),
                    Err(e) => tracing::error!("Error loading GPU static info: {}", e.to_string()),
                }
            }
            gpu_static
        })
        .0;

        let server = Self {
            system: sysinfo::System::new_all(),
            disks: sysinfo::Disks::new_with_refreshed_list(),
            cpu_static,
            mem_static,
            disk_static,
            gpu_static,
        };
        tracing::info!("Server initialized");

        let connection = zbus::connection::Builder::system()?
            .serve_at("/io/github/CosmicUtils/Observatory", server)?
            .name("io.github.CosmicUtils.Observatory")?
            .build()
            .await?;
        tracing::info!("monitord dbus created");

        let _ = sd_notify::notify(true, &[sd_notify::NotifyState::Ready]);

        loop {
            let server: zbus::object_server::InterfaceRef<SystemSnapshotServer> = connection
                .object_server()
                .interface("/io/github/CosmicUtils/Observatory")
                .await?;

            let system = &mut server.get_mut().await;
            let snapshot = system.load(backends.clone());

            server.snapshot(snapshot.await?).await?;

            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        Ok(())
    }

    pub(crate) async fn load(
        &mut self,
        backends: Vec<Arc<dyn gpu::GpuBackend>>,
    ) -> zbus::Result<SystemSnapshot> {
        self.disks.refresh(true);
        self.system.refresh_specifics(
            sysinfo::RefreshKind::nothing()
                .with_cpu(
                    sysinfo::CpuRefreshKind::nothing()
                        .with_cpu_usage()
                        .with_frequency(),
                )
                .with_memory(sysinfo::MemoryRefreshKind::nothing().with_ram().with_swap())
                .with_processes(
                    sysinfo::ProcessRefreshKind::nothing()
                        .with_cpu()
                        .with_memory()
                        .with_cmd(sysinfo::UpdateKind::OnlyIfNotSet)
                        .with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
                ),
        );
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.system
            .refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        let (processes, cpu_dynamic_info, mem_dynamic_info, disk_dynamic_info, gpu_dynamic_info) = tokio::join!(
            Process::load_all(&self.system),
            CpuDynamic::load(&self.system),
            MemoryDynamic::load(&self.system),
            DiskDynamic::load(&self.disks),
            async {
                let mut dynamics = Vec::new();
                for backend in backends.iter() {
                    match backend.get_dynamic() {
                        Ok(dynamic) => dynamics.extend(dynamic),
                        Err(e) => {
                            tracing::error!("Error loading GPU dynamic info: {}", e.to_string())
                        }
                    }
                }
                dynamics
            }
        );

        // Match up disks together
        let mut disks: Vec<(DiskStatic, DiskDynamic)> = Vec::new();
        for disk in disk_dynamic_info.iter() {
            if let Some(drive) = self
                .disk_static
                .iter()
                .find(|disk_static| disk.0.contains(disk_static.device.as_str()))
            {
                if let Some(existing_disk) =
                    disks.iter_mut().find(|disk| *disk.0.model == *drive.model)
                {
                    existing_disk.1.read += disk.1.read;
                    existing_disk.1.write += disk.1.write;
                } else {
                    disks.push((drive.clone(), disk.1.clone()));
                }
            } else {
                tracing::info!("Could not find a disk for device {}", disk.0);
            }
        }

        Ok(SystemSnapshot {
            processes,
            cpu: (self.cpu_static.clone(), cpu_dynamic_info),
            mem: (self.mem_static.clone(), mem_dynamic_info),
            disks,
            gpus: self
                .gpu_static
                .iter()
                .cloned()
                .zip(gpu_dynamic_info.iter().cloned())
                .collect(),
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
