use crate::app::message::AppMessage;
use crate::core::system_info::{GpuDynamicInfo, GpuStaticInfo, SystemInfo};
use crate::fl;
use cosmic::{app::Task, cosmic_theme, iced, theme, widget, Element};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

pub struct GpuResources {
    gpu_usage_histories: Vec<VecDeque<f32>>,

    gpu_dynamic_info: Vec<GpuDynamicInfo>,
    gpu_static_info: Vec<GpuStaticInfo>,
    sys: Arc<RwLock<SystemInfo>>,
}

impl GpuResources {
    pub fn new(sys: Arc<RwLock<SystemInfo>>) -> Self {
        Self {
            gpu_usage_histories: Vec::new(),
            gpu_dynamic_info: Vec::new(),
            gpu_static_info: Vec::new(),
            sys,
        }
    }

    fn graph(&self, cosmic: &theme::CosmicTheme) -> Element<AppMessage> {
        widget::layer_container(
            widget::column::with_children(
                self.gpu_usage_histories
                    .iter()
                    .map(|usage_history| {
                        widget::canvas(crate::widgets::line_graph::LineGraph {
                            steps: 59,
                            points: usage_history.clone(),
                            autoscale: false,
                        })
                        .height(iced::Length::Fill)
                        .width(iced::Length::Fill)
                        .into()
                    })
                    .collect(),
            )
            .spacing(cosmic.space_xs()),
        )
        .layer(cosmic_theme::Layer::Primary)
        .width(iced::Length::Fill)
        .into()
    }

    fn info_column(&self, cosmic: &theme::CosmicTheme) -> Element<AppMessage> {
        let mut gpus = Vec::new();
        for i in 0..self.gpu_dynamic_info.len() {
            gpus.push(
                widget::column()
                    .spacing(cosmic.space_s())
                    .push(widget::text::title4(
                        self.gpu_static_info[i].device_name.to_string(),
                    ))
                    .push(iced::widget::horizontal_rule(1))
                    // GPU Usage
                    .push(
                        widget::row()
                            .align_y(iced::Alignment::Center)
                            .push(widget::text::heading(fl!("utilization")))
                            .push(widget::horizontal_space())
                            .push(widget::text::heading(format!(
                                "{}%",
                                self.gpu_dynamic_info[i].util_percent
                            ))),
                    )
                    // VRAM
                    .push(
                        widget::row()
                            .align_y(iced::Alignment::Center)
                            .push(widget::text::heading(fl!("vram")))
                            .push(widget::horizontal_space())
                            .push(widget::text::heading(format!(
                                "{}%",
                                (self.gpu_dynamic_info[i].used_memory as f64
                                    / self.gpu_static_info[i].total_memory as f64
                                    * 100.0) as usize,
                            ))),
                    )
                    .into(),
            )
        }

        widget::layer_container(widget::column::with_children(gpus).spacing(cosmic.space_s()))
            .layer(cosmic_theme::Layer::Primary)
            .width(iced::Length::Fixed(280.))
            .height(iced::Length::Fill)
            .padding([cosmic.space_s(), cosmic.space_m()])
            .into()
    }
}

impl super::Page for GpuResources {
    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::SysInfoRefresh => {
                if let Ok(sys) = self.sys.read() {
                    self.gpu_dynamic_info = sys.gpu_dynamic_info();
                    for i in 0..self.gpu_dynamic_info.len() {
                        if self.gpu_usage_histories.len() == i {
                            self.gpu_usage_histories.push(VecDeque::from([0.0; 60]));
                        }
                        self.gpu_usage_histories[i]
                            .push_back(self.gpu_dynamic_info[i].util_percent as f32 / 100.0);
                        if self.gpu_usage_histories[i].len() > 60 {
                            self.gpu_usage_histories[i].pop_front();
                        }
                    }
                    self.gpu_static_info = sys.gpu_static_info();
                }
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<AppMessage> {
        let theme = theme::active();
        let cosmic = theme.cosmic();
        widget::layer_container(
            widget::row()
                .spacing(cosmic.space_xxs())
                .push(self.graph(&cosmic))
                .push(self.info_column(&cosmic)),
        )
        .layer(cosmic_theme::Layer::Background)
        .into()
    }
}
