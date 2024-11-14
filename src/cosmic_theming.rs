use cosmic::iced::Color;

pub fn button_style(selected: bool, use_accent: bool) -> cosmic::theme::Button {
    cosmic::theme::Button::Custom {
        active: Box::new(move |focused, theme|
            button_appearance(&theme, selected, false, focused, use_accent)),
        disabled: Box::new(move |theme|
            button_appearance(&theme, selected, false, false, use_accent)),
        hovered: Box::new(move |focused, theme|
            button_appearance(&theme, selected, true, focused, use_accent)),
        pressed: Box::new(move |focused, theme|
            button_appearance(&theme, selected, true, focused, use_accent)),
    }
}


fn button_appearance(theme: &cosmic::theme::Theme, selected: bool, hovered: bool, focused: bool, use_accent: bool) -> cosmic::widget::button::Style {
    let cosmic = theme.cosmic();
    let mut appearance = cosmic::widget::button::Style::default();

    if selected {
        // Appearance for when the button is selected
        if use_accent {
            appearance.background = Some(Color::from(cosmic.accent_color()).into());
            appearance.icon_color = Some(Color::from(cosmic.on_accent_color()));
            appearance.text_color = Some(Color::from(cosmic.on_accent_color()));
        } else {
            appearance.background = Some(Color::from(cosmic.bg_component_color()).into());
        }
    } else if hovered {
        // Appearance for when the button is hovered
        appearance.background = Some(Color::from(cosmic.bg_component_color()).into());
        appearance.icon_color = Some(Color::from(cosmic.on_bg_component_color()));
        appearance.text_color = Some(Color::from(cosmic.on_bg_component_color()));
    }

    if focused && use_accent {
        appearance.outline_width = 1.0;
        appearance.outline_color = Color::from(cosmic.accent_color());
        appearance.border_width = 2.0;
        appearance.border_color = Color::TRANSPARENT;
    }
    appearance.border_radius = cosmic.radius_xs().into();

    appearance
}