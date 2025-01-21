use cosmic::{app::Task, prelude::*, widget};
use monitord::system::{cpu::CpuDynamic, CpuStatic};

use crate::app::Message;

pub struct CpuPage {
    cpu_info: Option<CpuStatic>,
    cpu_dyn: Option<CpuDynamic>,
}

impl CpuPage {
    pub fn new() -> Self {
        Self {
            cpu_info: None,
            cpu_dyn: None,
        }
    }
}

impl super::super::Page for CpuPage {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Snapshot(snapshot) => {
                self.cpu_info = Some(snapshot.cpu_static_info.clone());
                self.cpu_dyn = Some(snapshot.cpu_dynamic_info.clone());
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let theme = cosmic::theme::active();
        let cosmic = theme.cosmic();
        widget::column()
            .spacing(cosmic.space_m())
            .push(
                widget::settings::section()
                    .title("Information")
                    .add(widget::settings::item(
                        "Model",
                        widget::text::caption(
                            self.cpu_info
                                .as_ref()
                                .map(|cpuinf| cpuinf.model.clone())
                                .unwrap_or("Not Loaded".to_string()),
                        ),
                    ))
                    .add(widget::settings::item(
                        "Physical Cores",
                        widget::text::body(
                            self.cpu_info
                                .as_ref()
                                .map(|cpuinf| cpuinf.physical_cores.to_string())
                                .unwrap_or("Not Loaded".to_string()),
                        ),
                    ))
                    .add(widget::settings::item(
                        "Logical Cores",
                        widget::text::body(
                            self.cpu_info
                                .as_ref()
                                .map(|cpuinf| cpuinf.logical_cores.to_string())
                                .unwrap_or("Not Loaded".to_string()),
                        ),
                    )),
            )
            .push(
                widget::settings::section()
                    .title("Statistics")
                    .add(widget::settings::item(
                        "Speed",
                        widget::text::body(
                            self.cpu_dyn
                                .as_ref()
                                .map(|cpudyn| format!("{:.2} GHz", cpudyn.speed as f32 / 1000.0))
                                .unwrap_or("Not Loaded".to_string()),
                        ),
                    ))
                    .add(widget::settings::item(
                        "Usage",
                        widget::text::body(
                            self.cpu_dyn
                                .as_ref()
                                .map(|cpudyn| format!("{}%", cpudyn.usage.round()))
                                .unwrap_or("Not Loaded".to_string()),
                        ),
                    )),
            )
            .apply(Element::from)
    }
}
