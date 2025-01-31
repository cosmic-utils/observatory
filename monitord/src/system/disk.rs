use std::path::Path;

#[derive(zbus::zvariant::Type, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DiskStatic {
    pub model: String,
    pub device: String,
    pub size: u64,
}

impl DiskStatic {
    pub(crate) async fn load() -> Vec<Self> {
        let client = udisks2::Client::new().await.expect("Udisks2 not loaded!");
        tracing::info!("Connected to UDisks2 for disk information");

        let block_devices = client
            .manager()
            .get_block_devices(udisks2::standard_options(false))
            .await
            .expect("Could not get block devices");

        let mut disks = Vec::new();

        for block_path in block_devices {
            if client
                .object(block_path.clone())
                .unwrap()
                .partition()
                .await
                .is_err()
                && client
                    .object(block_path.clone())
                    .unwrap()
                    .r#loop()
                    .await
                    .is_err()
                && client
                    .object(block_path.clone())
                    .unwrap()
                    .swapspace()
                    .await
                    .is_err()
            {
                let block = client
                    .object(block_path.clone())
                    .unwrap()
                    .block()
                    .await
                    .unwrap();
                let disk_path = block.drive().await.unwrap();
                if disk_path.as_str() != "/" {
                    let drive = client.object(disk_path).unwrap().drive().await.unwrap();

                    disks.push(DiskStatic {
                        model: drive.model().await.unwrap(),
                        device: Path::new(block_path.as_str())
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .into(),
                        size: block.size().await.unwrap(),
                    });
                    tracing::info!("Read static drive info: {:?}", disks.last().unwrap());
                }
            }
        }

        disks
    }
}
