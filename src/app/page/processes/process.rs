use crate::{fl, helpers};
use cosmic::{iced::Length, widget};
use lazy_static::lazy_static;
use monitord_protocols::monitord::ProcessInfo;
use std::borrow::Cow;

lazy_static! {
    static ref PROC_NAME: String = fl!("name");
    static ref PROC_CPU: String = fl!("cpu");
    static ref PROC_GPU: String = fl!("gpu");
    static ref PROC_MEM: String = fl!("mem");
    static ref PROC_DISK: String = fl!("disk");
}

pub struct ProcessTableItem {
    pub process: ProcessInfo,
    name: Cow<'static, str>,
    cpu: Cow<'static, str>,
    gpu: Cow<'static, str>,
    mem: Cow<'static, str>,
    disk: Cow<'static, str>,
}

impl ProcessTableItem {
    pub fn new(process: ProcessInfo) -> Self {
        Self {
            name: process.name.clone().into(),
            cpu: format!("{}%", process.cpu_usage_percent.round()).into(),
            gpu: format!(
                "{}%",
                process
                    .gpu_usage
                    .as_ref()
                    .map(|gpu| gpu.gpu_utilization_percent)
                    .unwrap_or_default()
                    .round()
            )
            .into(),
            mem: helpers::get_bytes(process.physical_memory_bytes).into(),
            disk: format!(
                "{}/s",
                helpers::get_bytes(
                    process.disk_read_bytes_per_sec + process.disk_write_bytes_per_sec
                )
            )
            .into(),
            process,
        }
    }
}

impl widget::table::ItemInterface<ProcessTableCategory> for ProcessTableItem {
    fn get_icon(&self, category: ProcessTableCategory) -> Option<widget::Icon> {
        match category {
            ProcessTableCategory::Name => {
                Some(widget::icon::from_name("application-default-symbolic").icon())
            }
            _ => None,
        }
    }

    fn get_text(&self, category: ProcessTableCategory) -> Cow<'static, str> {
        match category {
            ProcessTableCategory::Name => self.name.clone(),
            ProcessTableCategory::Cpu => self.cpu.clone(),
            ProcessTableCategory::Gpu => self.gpu.clone(),
            ProcessTableCategory::Mem => self.mem.clone(),
            ProcessTableCategory::Disk => self.disk.clone(),
        }
    }

    fn compare(&self, other: &Self, category: ProcessTableCategory) -> std::cmp::Ordering {
        let self_gpu = self
            .process
            .gpu_usage
            .as_ref()
            .map(|usage| usage.gpu_utilization_percent)
            .unwrap_or_default();
        let other_gpu = other
            .process
            .gpu_usage
            .as_ref()
            .map(|usage| usage.gpu_utilization_percent)
            .unwrap_or_default();

        let self_disk =
            self.process.disk_read_bytes_per_sec + self.process.disk_write_bytes_per_sec;
        let other_disk =
            other.process.disk_read_bytes_per_sec + other.process.disk_write_bytes_per_sec;

        match category {
            ProcessTableCategory::Name => other
                .name
                .to_ascii_lowercase()
                .cmp(&self.name.to_ascii_lowercase()),
            ProcessTableCategory::Cpu => self
                .process
                .cpu_usage_percent
                .partial_cmp(&other.process.cpu_usage_percent)
                .unwrap(),
            ProcessTableCategory::Gpu => self_gpu.partial_cmp(&other_gpu).unwrap(),
            ProcessTableCategory::Mem => self
                .process
                .physical_memory_bytes
                .cmp(&other.process.physical_memory_bytes),
            ProcessTableCategory::Disk => self_disk.cmp(&other_disk),
        }
    }
}

#[derive(Default, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum ProcessTableCategory {
    #[default]
    Name,
    Cpu,
    Gpu,
    Mem,
    Disk,
}

impl std::fmt::Display for ProcessTableCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Name => PROC_NAME.as_str(),
                Self::Cpu => PROC_CPU.as_str(),
                Self::Gpu => PROC_GPU.as_str(),
                Self::Mem => PROC_MEM.as_str(),
                Self::Disk => PROC_DISK.as_str(),
            }
        )
    }
}

impl widget::table::ItemCategory for ProcessTableCategory {
    fn width(&self) -> cosmic::iced::Length {
        match self {
            Self::Name => Length::Fixed(320.0),
            Self::Cpu => Length::Fixed(100.0),
            Self::Gpu => Length::Fixed(100.0),
            Self::Mem => Length::Fixed(120.0),
            Self::Disk => Length::Fixed(150.0),
        }
    }
}
