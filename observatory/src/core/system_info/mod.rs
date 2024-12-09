pub mod mem_info;
//mod net_info;
pub mod dbus_interface;
pub mod proc_info;

use dbus::blocking::{
    stdintf::{org_freedesktop_dbus::Peer, org_freedesktop_dbus::Properties},
    LocalConnection, Proxy,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

pub use dbus_interface::*;

macro_rules! dbus_call {
    ($self: ident, $method: tt, $dbus_method_name: literal $(,$args:ident)*) => {{
        use dbus_interface::Gatherer;

        const RETRY_COUNT: i32 = 10;

        for i in 1..=RETRY_COUNT {
            match $self.proxy.$method($($args,)*) {
                Ok(reply) => {
                    return reply;
                }
                Err(e) => {
                    match $self.is_running() {
                        Ok(()) => {
                            if e.name() == Some("org.freedesktop.DBus.Error.NoReply") {
                                log::error!(
                                    "DBus call '{}' timed out, on try {}",
                                    $dbus_method_name, i,
                                );

                                if i == RETRY_COUNT - 1 {
                                    log::error!("Restarting Daemon...");
                                    $self.stop();
                                    $self.start();
                                } else {
                                    std::thread::sleep(Duration::from_millis(100));
                                }
                            } else {
                                log::error!(
                                    "DBus call '{}' failed on try {}: {}",
                                    $dbus_method_name, i, e,
                                );

                                std::thread::sleep(Duration::from_millis(100));
                            }
                        }
                        Err(exit_code) => {
                            log::error!(
                                "Child failed, on try {}, with exit code {}. Restarting Gatherer...",
                                i, exit_code,
                            );
                            $self.start();
                        }
                    }
                }
            }
        }

        panic!();
    }};
}

pub struct SystemInfo {
    #[allow(dead_code)]
    connection: Rc<LocalConnection>,
    proxy: Proxy<'static, Rc<LocalConnection>>,

    child: RefCell<Option<std::process::Child>>,
}

impl Debug for SystemInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemInfo").finish()
    }
}

impl PartialEq for SystemInfo {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl SystemInfo {
    pub fn new() -> Self {
        let connection = Rc::new(LocalConnection::new_session().unwrap_or_else(|e| {
            log::error!("Failed to connect to D-Bus: {}", e.to_string());
            panic!();
        }));
        let proxy = Proxy::new(
            OD_INTERFACE_NAME,
            OD_OBJECT_PATH,
            Duration::from_millis(1000),
            connection.clone(),
        );

        Self {
            connection,
            proxy,
            child: RefCell::new(None),
        }
    }

    pub fn start(&self) {
        let mut command = {
            let mut cmd = std::process::Command::new(Self::executable());
            cmd.env_remove("LD_PRELOAD");

            if let Some(mut appdir) = std::env::var_os("APPDIR") {
                appdir.push("/runtime/default");
                cmd.current_dir(appdir);
            }

            cmd
        };
        command
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit());

        self.child.borrow_mut().replace(match command.spawn() {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to spawn Gatherer process: {}", &e);
                panic!();
            }
        });

        const START_WAIT_TIME_MS: u64 = 300;
        const RETRY_COUNT: i32 = 50;

        // Let the child process start up
        for i in 0..RETRY_COUNT {
            std::thread::sleep(Duration::from_millis(START_WAIT_TIME_MS / 2));
            match self.proxy.ping() {
                Ok(()) => return,
                Err(e) => {
                    log::error!("Call to Gatherer Ping method failed on try {}: {}", i, e,);
                }
            }
            std::thread::sleep(Duration::from_millis(START_WAIT_TIME_MS / 2));
        }

        panic!("Failed to spawn Gatherer process: Did not respond to Ping");
    }

    pub fn stop(&self) {
        let child = self.child.borrow_mut().take();
        if let Some(mut child) = child {
            // Try to get the child to wake up in case it's stuck
            #[cfg(target_family = "unix")]
            unsafe {
                libc::kill(child.id() as _, libc::SIGCONT);
            }

            let _ = child.kill();
            for _ in 0..2 {
                match child.try_wait() {
                    Ok(Some(_)) => return,
                    Ok(None) => {
                        // Wait a bit and try again, the child process might just be slow to stop
                        std::thread::sleep(Duration::from_millis(20));
                        continue;
                    }
                    Err(e) => {
                        log::error!("Failed to wait for Gatherer process to stop: {}", &e);

                        panic!();
                    }
                }
            }
        }
    }

    pub fn is_running(&self) -> Result<(), i32> {
        let mut lock = self.child.borrow_mut();

        let child = match lock.as_mut() {
            Some(child) => child,
            None => return Err(-1),
        };

        let status = match child.try_wait() {
            Ok(None) => return Ok(()),
            Ok(Some(status)) => status,
            Err(_) => {
                return Err(-1);
            }
        };

        match status.code() {
            Some(status_code) => Err(status_code),
            None => Err(-1),
        }
    }

    fn executable() -> String {
        let exe_simple = "observatory-daemon".to_owned();

        log::debug!("Gatherer executable name: {}", &exe_simple);

        exe_simple
    }
}

#[allow(dead_code)]
impl SystemInfo {
    pub fn set_refresh_interval(&self, interval: u64) {
        if let Err(e) = self
            .proxy
            .set(OD_INTERFACE_NAME, "RefreshInterval", interval)
        {
            log::error!("Failed to set RefreshInterval property: {e}");
        }
    }

    pub fn set_core_count_affects_percentages(&self, v: bool) {
        if let Err(e) = self
            .proxy
            .set(OD_INTERFACE_NAME, "CoreCountAffectsPercentages", v)
        {
            log::error!("Failed to set CoreCountAffectsPercentages property: {e}");
        }
    }

    pub fn cpu_static_info(&self) -> CpuStaticInfo {
        dbus_call!(self, get_cpu_static_info, "GetCPUStaticInfo");
    }

    pub fn cpu_dynamic_info(&self) -> CpuDynamicInfo {
        dbus_call!(self, get_cpu_dynamic_info, "GetCPUDynamicInfo");
    }

    pub fn disks_info(&self) -> Vec<DiskInfo> {
        dbus_call!(self, get_disks_info, "GetDisksInfo");
    }

    pub fn fans_info(&self) -> Vec<FanInfo> {
        dbus_call!(self, get_fans_info, "GetFansInfo");
    }

    #[allow(unused)]
    pub fn gpu_list(&self) -> Vec<Arc<str>> {
        dbus_call!(self, get_gpu_list, "GetGPUList");
    }

    pub fn gpu_static_info(&self) -> Vec<GpuStaticInfo> {
        dbus_call!(self, get_gpu_static_info, "GetGPUStaticInfo");
    }

    pub fn gpu_dynamic_info(&self) -> Vec<GpuDynamicInfo> {
        dbus_call!(self, get_gpu_dynamic_info, "GetGPUDynamicInfo");
    }

    pub fn processes(&self) -> HashMap<u32, Process> {
        dbus_call!(self, get_processes, "GetProcesses");
    }

    pub fn apps(&self) -> HashMap<Arc<str>, App> {
        dbus_call!(self, get_apps, "GetApps");
    }

    pub fn services(&self) -> HashMap<Arc<str>, Service> {
        dbus_call!(self, get_services, "GetServices");
    }

    pub fn terminate_process(&self, pid: u32) {
        dbus_call!(self, terminate_process, "TerminateProcess", pid);
    }

    pub fn kill_process(&self, pid: u32) {
        dbus_call!(self, kill_process, "KillProcess", pid);
    }

    pub fn start_service(&self, service_name: &str) {
        dbus_call!(self, start_service, "StartService", service_name);
    }

    pub fn stop_service(&self, service_name: &str) {
        dbus_call!(self, stop_service, "StopService", service_name);
    }

    pub fn restart_service(&self, service_name: &str) {
        dbus_call!(self, restart_service, "RestartService", service_name);
    }

    pub fn enable_service(&self, service_name: &str) {
        dbus_call!(self, enable_service, "EnableService", service_name);
    }

    pub fn disable_service(&self, service_name: &str) {
        dbus_call!(self, disable_service, "DisableService", service_name);
    }

    pub fn get_service_logs(&self, service_name: &str, pid: Option<NonZeroU32>) -> Arc<str> {
        dbus_call!(self, get_service_logs, "GetServiceLogs", service_name, pid);
    }
}
