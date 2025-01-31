mod system;

#[tokio::main]
async fn main() -> zbus::Result<()> {
    tracing_subscriber::fmt().init();
    tracing::info!("Logging Initialized");

    system::SystemSnapshotServer::run().await?;
    Ok(())
}
