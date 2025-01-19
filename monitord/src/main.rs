mod system;
use system::SystemServerSignals;

#[tokio::main]
async fn main() -> zbus::Result<()> {
    tracing_subscriber::fmt().init();

    let mut system_server = system::SystemServer::new();
    let mut system = sysinfo::System::new_all();
    let connection = zbus::connection::Builder::session()?
        .name("io.github.CosmicUtils.Observatory")?
        .serve_at("/io/github/CosmicUtils/Observatory", system_server)?
        .build()
        .await?;
    tracing::info!("Observatory dbus server set up");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));

        let snapshot = system::SystemSnapshot::load(&mut system);

        connection
            .object_server()
            .interface("/io/github/CosmicUtils/Observatory")
            .await?
            .snapshot(snapshot)
            .await?;
    }
}
