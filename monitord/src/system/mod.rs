pub mod process;
pub use process::Process;

#[derive(zbus::zvariant::Type, serde::Serialize, serde::Deserialize)]
pub struct SystemSnapshot {
    pub processes: Vec<Process>,
}

#[zbus::proxy(
    interface = "io.github.CosmicUtils.Observatory.SystemSnapshot",
    default_service = "io.github.CosmicUtils.Observatory",
    default_path = "/io/github/CosmicUtils/Observatory"
)]
trait SystemSnapshot {
    fn snapshot(&self, instance: SystemSnapshot) -> zbus::Result<()>;
}

pub struct SystemServer {}

#[zbus::interface(name = "io.github.CosmicUtils.Observatory.SystemSnapshot")]
impl SystemServer {
    #[zbus(signal)]
    async fn snapshot(
        signal_emitter: &zbus::object_server::SignalEmitter<'_>,
        instance: SystemSnapshot,
    ) -> zbus::Result<()>;
}
