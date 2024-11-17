use cosmic::iced::{Degrees, Rectangle, Renderer, Vector};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::widget::canvas;
use cosmic::Theme;
use cosmic::iced::mouse::Cursor;
use cosmic::iced_core::text::{LineHeight, Shaping};
use cosmic::iced_widget::canvas::Geometry;

#[derive(Debug)]
pub struct Meter {
    pub percentage: f32,
    pub thickness: f32,
}

impl canvas::Program<crate::app::message::Message, Theme> for Meter {
    type State = ();

    fn draw(&self, _state: &Self::State, renderer: &Renderer, theme: &Theme, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry<Renderer>> {
        let cosmic = theme.cosmic();
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let scale = frame.width().min(frame.height()) / 200.;
        let true_radius = frame.width().min(frame.height()) / 2.0 - self.thickness / 2.0;

        let mut bg_arc = canvas::path::Builder::new();
        bg_arc.arc(canvas::path::Arc {
            center: frame.center(),
            radius: true_radius,
            start_angle: Degrees(-150.).into(),
            end_angle: Degrees(-30.).into(),
        });
        frame.stroke(&bg_arc.build(), canvas::Stroke {
            style: canvas::stroke::Style::Solid(cosmic.bg_component_color().into()),
            width: self.thickness * scale,
            line_cap: canvas::LineCap::Round,
            ..Default::default()
        });

        let end_angle = self.percentage * 120.;

        let mut arc = canvas::path::Builder::new();
        arc.arc(canvas::path::Arc {
            center: frame.center(),
            radius: true_radius,
            start_angle: Degrees(-150.).into(),
            end_angle: Degrees(-150. + end_angle).into(),
        });
        frame.stroke(&arc.build(), canvas::Stroke {
            style: canvas::stroke::Style::Solid(cosmic.accent_color().into()),
            width: self.thickness * scale,
            line_cap: canvas::LineCap::Round,
            ..Default::default()
        });

        let text = canvas::Text {
            content: format!("{}%", get_percentage(self.percentage)),
            position: frame.center() - Vector::new(0., true_radius * 0.25),
            color: cosmic.on_bg_color().into(),
            size: (30. * scale).into(),
            line_height: LineHeight::Relative(1.),
            font: cosmic::font::default(),
            horizontal_alignment: Horizontal::Center,
            vertical_alignment: Vertical::Center,
            shaping: Shaping::Basic,
        };
        frame.fill_text(text);

        vec![frame.into_geometry()]
    }
}

fn get_percentage(percentage: f32) -> f32 {
    (percentage * 100.).round()
}