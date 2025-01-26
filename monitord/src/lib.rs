pub mod system;

#[derive(Clone, Debug)]
pub struct Interface<'a> {
    proxy: system::SystemSnapshotProxy<'a>,
}

impl<'a> Interface<'a> {
    pub async fn init() -> zbus::Result<Self> {
        let connection = zbus::Connection::session().await?;
        let proxy = system::SystemSnapshotProxy::builder(&connection)
            .build()
            .await?;

        Ok(Self { proxy })
    }

    pub async fn kill_process(&self, pid: u32) -> zbus::Result<bool> {
        Ok(self.proxy.kill_process(pid).await?)
    }

    pub async fn term_process(&self, pid: u32) -> zbus::Result<bool> {
        Ok(self.proxy.term_process(pid).await?)
    }

    pub async fn get_signal_iter(&self) -> zbus::Result<system::SnapshotStream> {
        Ok(self.proxy.receive_snapshot().await?)
    }
}

pub fn run() {
    /*if IS_FLATPAK {
        todo!()
    } */
    todo!()
}
