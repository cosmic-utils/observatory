mod system;
use system::SystemServerSignals;

#[tokio::main]
async fn main() -> zbus::Result<()> {
    let system_server = system::SystemServer {};
    let connection = zbus::connection::Builder::session()?
        .name("io.github.CosmicUtils.Observatory")?
        .serve_at("/io/github/CosmicUtils/Observatory", system_server)?
        .build()
        .await?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        connection
            .object_server()
            .interface("/io/github/CosmicUtils/Observatory")
            .await?
            .snapshot(system::SystemSnapshot {
                processes: Vec::new(),
            })
            .await?;
    }
}
