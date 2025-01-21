use std::{ffi::OsStr, path::Path};

#[derive(zbus::zvariant::Type, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Process {
    /// The internal Process ID
    pub pid: u32,
    /// The process's name for displaying
    pub displayname: String,

    /// The process's "real" name
    pub name: String,
    /// The command line of the process (if it was called with a symlink this is correct)
    pub cmd: Vec<String>,
    /// The executable of the process
    pub exe: String,

    /// CPU Usage (per-core)
    pub cpu: f32,
    /// GPU Usage (per-gpu)
    pub gpu: Vec<f32>,
    /// Memory usage (in bytes)
    pub memory: u64,
    /// Total disk usage (read and write)
    pub disk: u64,
}

impl Process {
    pub(crate) fn load_all(system: &sysinfo::System) -> Vec<Process> {
        let mut processes = Vec::new();

        for (pid, proc) in system.processes() {
            if proc.thread_kind().is_none() {
                let mut process = Process {
                    pid: pid.as_u32(),
                    displayname: String::new(),
                    name: proc.name().to_string_lossy().into(),
                    cmd: proc
                        .cmd()
                        .iter()
                        .map(|cmd| cmd.to_string_lossy().to_string())
                        .collect::<Vec<String>>(),
                    exe: proc.exe().unwrap_or(Path::new("")).to_string_lossy().into(),
                    cpu: proc.cpu_usage(),
                    gpu: vec![0.0],
                    memory: proc.memory(),
                    disk: proc.disk_usage().read_bytes + proc.disk_usage().written_bytes,
                };
                process.displayname = if !process.cmd.is_empty() {
                    let cmd = process.cmd.join(" ");
                    let path = Path::new(cmd.split(" ").nth(0).unwrap_or(""));
                    path.file_name()
                        .unwrap_or(OsStr::new(process.name.as_str()))
                        .to_string_lossy()
                        .to_string()
                } else if !process.exe.is_empty() {
                    Path::new(&process.exe)
                        .file_name()
                        .map(|filename| filename.to_string_lossy().to_string())
                        .unwrap_or(process.name.clone())
                } else {
                    process.name.clone()
                };
                processes.push(process);
            }
        }

        processes
    }
}
