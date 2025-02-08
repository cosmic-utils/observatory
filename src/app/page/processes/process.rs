use crate::{fl, helpers};
use cosmic::{iced::Length, widget};
use lazy_static::lazy_static;
use monitord::system::Process;
use std::borrow::Cow;

lazy_static! {
    static ref PROC_NAME: String = fl!("name");
    static ref PROC_CPU: String = fl!("cpu");
    static ref PROC_GPU0: String = format!("{} 0", fl!("gpu"));
    static ref PROC_GPU1: String = format!("{} 1", fl!("gpu"));
    static ref PROC_MEM: String = fl!("mem");
    static ref PROC_DISK: String = fl!("disk");
}

pub struct ProcessTableItem {
    pub process: Process,
    name: Cow<'static, str>,
    cpu: Cow<'static, str>,
    gpu: Vec<Cow<'static, str>>,
    mem: Cow<'static, str>,
    disk: Cow<'static, str>,
}

impl ProcessTableItem {
    pub fn new(process: Process) -> Self {
        Self {
            name: process.displayname.clone().into(),
            cpu: format!("{}%", process.cpu.round()).into(),
            gpu: process
                .gpu
                .iter()
                .map(|usage| format!("{}%", usage.round()).into())
                .collect::<Vec<Cow<str>>>(),
            mem: helpers::get_bytes(process.memory).into(),
            disk: format!("{}/s", helpers::get_bytes(process.disk)).into(),
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
            ProcessTableCategory::Gpu(num) => self.gpu.get(num as usize).unwrap().clone(),
            ProcessTableCategory::Mem => self.mem.clone(),
            ProcessTableCategory::Disk => self.disk.clone(),
        }
    }

    fn compare(&self, other: &Self, category: ProcessTableCategory) -> std::cmp::Ordering {
        match category {
            ProcessTableCategory::Name => other
                .name
                .to_ascii_lowercase()
                .cmp(&self.name.to_ascii_lowercase()),
            ProcessTableCategory::Cpu => self.process.cpu.partial_cmp(&other.process.cpu).unwrap(),
            ProcessTableCategory::Gpu(num) => self.process.gpu[num as usize]
                .partial_cmp(&other.process.gpu[num as usize])
                .unwrap(),
            ProcessTableCategory::Mem => self.process.memory.cmp(&other.process.memory),
            ProcessTableCategory::Disk => self.process.disk.cmp(&other.process.disk),
        }
    }
}

#[derive(Default, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum ProcessTableCategory {
    #[default]
    Name,
    Cpu,
    Gpu(u16),
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
                Self::Gpu(num) => match num {
                    0 => PROC_GPU0.as_str(),
                    1 => PROC_GPU1.as_str(),
                    _ => unreachable!(),
                },
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
            Self::Cpu => Length::Fixed(80.0),
            Self::Gpu(_) => Length::Fixed(80.0),
            Self::Mem => Length::Fixed(120.0),
            Self::Disk => Length::Fixed(150.0),
        }
    }
}
