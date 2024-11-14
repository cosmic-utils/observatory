use cosmic::iced::{Point, Rectangle, Renderer};
use cosmic::iced::mouse::Cursor;
use cosmic::iced_widget::canvas::Geometry;
use cosmic::widget::canvas;
use cosmic::widget::canvas::path::Builder;
use cosmic::widget::canvas::{LineCap, LineJoin, Path, Stroke, Style};
use cosmic::theme;
use std::collections::VecDeque;
use cosmic::widget::canvas::Gradient;
use cosmic::widget::canvas::gradient::Linear;

#[derive(Debug)]
pub struct LineGraph {
    pub steps: usize,
    pub points: VecDeque<f32>,
}

impl canvas::Program<crate::app::message::Message, theme::Theme> for LineGraph {
    type State = ();
    fn draw(&self, state: &Self::State, renderer: &Renderer, theme: &theme::Theme, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry<Renderer>> {
        let cosmic = theme.cosmic();
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let right = frame.center().x + frame.width() / 2.0;
        let bottom = frame.center().y + frame.height() / 2.0;

        // Background
        //    Outer square
        let mut builder = Builder::new();
        builder.move_to(Point::new(right, bottom));
        builder.line_to(Point::new(right, bottom - frame.height()));
        builder.line_to(Point::new(right - frame.height(), bottom - frame.height()));
        builder.line_to(Point::new(right - frame.height(), bottom));
        builder.line_to(Point::new(right, bottom));
        frame.stroke(&builder.build(), Stroke {
            style: Style::Solid(cosmic.bg_component_color().into()),
            width: 2.0,
            ..Default::default()
        });

        // Build the line graph
        let x_step = frame.width() / self.steps as f32;
        let gradient = Linear::new(Point::new(right, bottom - frame.height()), Point::new(right, bottom))
            .add_stop(0.0, cosmic.accent_color().into())
            .add_stop(0.5, cosmic.accent_color().into())
            .add_stop(1.0, {
                let mut color = cosmic.accent_color();
                color.alpha = 0.0;
                color
            }.into());
        for i in 0..self.points.len() {
            let level_point = Point::new(right - i as f32 * x_step, bottom - frame.height() * self.points[i]);
            let bottom_point = Point::new(right - i as f32 * x_step, bottom);
            let line = Path::line(level_point, bottom_point);

            let stroke = Stroke {
                line_cap: LineCap::Round,
                style: Style::Gradient(Gradient::Linear(gradient)),
                width: 5.0,
                ..Default::default()
            };
            frame.stroke(&line, stroke);
        }


        vec![frame.into_geometry()]
    }
}