use cosmic::{iced, prelude::*, widget::canvas::*};

#[derive(Clone)]
pub struct LineGraph {
    // Points (out of 1)
    pub points: Vec<f32>,
}

impl Program<crate::app::Message, Theme> for LineGraph {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::core::mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let cosmic = theme.cosmic();

        let bounds = iced::Rectangle::new(
            iced::Point::new(0.0, 0.0),
            iced::Size::new(
                bounds.width.min(bounds.height),
                bounds.height.min(bounds.width),
            ),
        );
        let mut frame = Frame::new(renderer, bounds.size());

        // Draw background grid
        let mut builder = path::Builder::new();
        let x_step = bounds.width / 10.0;
        let y_step = bounds.height / 10.0;
        for i in 1..10 {
            // Vertical line
            builder.move_to(iced::Point::new(x_step * i as f32, bounds.y));
            builder.line_to(iced::Point::new(
                x_step * i as f32,
                bounds.y + bounds.height,
            ));
            // Horizontal line
            builder.move_to(iced::Point::new(bounds.x, y_step * i as f32));
            builder.line_to(iced::Point::new(bounds.x + bounds.width, y_step * i as f32));
        }
        frame.stroke(
            &builder.build(),
            Stroke {
                style: Style::Solid(cosmic.bg_divider().into()),
                width: 1.0,
                ..Default::default()
            },
        );

        // Draw points
        let bounds = bounds.shrink(1.0);
        let x_step = bounds.width / (self.points.len() as f32 - 1.0);

        let mut builder = path::Builder::new();
        let mut current_pos = bounds.position() + iced::Vector::new(0.0, bounds.height);
        builder.move_to(current_pos);
        for (index, point) in self.points.iter().enumerate() {
            let x = index as f32 * x_step;
            let y = (bounds.y + bounds.height) - point * bounds.height;
            let control = x - (x_step * 0.5);
            builder.bezier_curve_to(
                iced::Point::new(control, current_pos.y),
                iced::Point::new(control, y),
                iced::Point::new(x, y),
            );
            current_pos = iced::Point::new(x, y);
        }
        builder.line_to(iced::Point::new(
            bounds.x + bounds.width,
            bounds.y + bounds.height,
        ));
        builder.close();
        let path = builder.build();
        frame.stroke(
            &path,
            Stroke {
                style: Style::Solid(cosmic.accent_color().into()),
                width: 2.0,
                ..Default::default()
            },
        );
        frame.fill(
            &path,
            Fill {
                style: Style::Solid(
                    cosmic
                        .accent_color()
                        .apply(|mut color| {
                            color.alpha = 0.25;
                            color
                        })
                        .into(),
                ),
                ..Default::default()
            },
        );

        // This is bs but it works, draw a background colored rounded rectangle to "hide" things drawing outside of the graph
        let mut square = path::Builder::new();
        let ex_bounds = bounds.expand(8.0);
        square.rounded_rectangle(
            ex_bounds.position(),
            ex_bounds.size(),
            cosmic.radius_l().into(),
        );
        frame.stroke(
            &square.build(),
            Stroke {
                style: Style::Solid(cosmic.bg_color().into()),
                width: 12.0,
                ..Default::default()
            },
        );
        // Draw background square
        let mut square = path::Builder::new();
        square.rounded_rectangle(bounds.position(), bounds.size(), cosmic.radius_m().into());
        frame.stroke(
            &square.build(),
            Stroke {
                style: Style::Solid(cosmic.accent_color().into()),
                width: 2.0,
                ..Default::default()
            },
        );

        vec![frame.into_geometry()]
    }
}
