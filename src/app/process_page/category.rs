use crate::app::message::Message;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::Length;
use cosmic::{theme, widget, Element};

pub struct CategoryList(pub Vec<Category>);

#[derive(PartialEq)]
pub enum Sort {
    Ascending,
    Descending,
}

impl Sort {
    pub fn opposite(&mut self) {
        if *self == Sort::Ascending {
            *self = Sort::Descending;
        } else {
            *self = Sort::Ascending
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Category {
    Name,
    User,
    Cpu,
    Memory,
    Disk,
}

impl CategoryList {
    pub fn new() -> Self {
        CategoryList(
            [
                Category::Name,
                Category::User,
                Category::Cpu,
                Category::Memory,
                Category::Disk,
            ]
            .into(),
        )
    }

    pub fn element(&self, theme: &theme::Theme, sort: &(Category, Sort)) -> Element<Message> {
        let row = widget::row::with_children(
            self.0
                .iter()
                .map(|category| category.element(&theme, sort))
                .collect(),
        );

        widget::container(row).into()
    }
}

impl Category {
    pub fn name(&self) -> String {
        match self {
            Category::Name => "Name",
            Category::User => "User",
            Category::Cpu => "CPU",
            Category::Memory => "Memory",
            Category::Disk => "Disk",
        }
        .into()
    }

    pub fn width(&self) -> Length {
        match self {
            Category::Name => Length::Fixed(300.),
            Category::User => Length::Fixed(80.),
            Category::Cpu => Length::Fixed(70.),
            Category::Memory => Length::Fixed(100.),
            Category::Disk => Length::Fixed(120.),
        }
    }

    pub fn alignment(&self) -> Horizontal {
        match self {
            Category::Name => Horizontal::Left,
            Category::User => Horizontal::Left,
            Category::Cpu => Horizontal::Right,
            Category::Memory => Horizontal::Right,
            Category::Disk => Horizontal::Right,
        }
    }

    pub fn index(&self) -> u8 {
        match self {
            Category::Name => 0,
            Category::User => 1,
            Category::Cpu => 2,
            Category::Memory => 3,
            Category::Disk => 4,
        }
    }

    pub fn from_index(index: u8) -> Category {
        match index {
            0 => Category::Name,
            1 => Category::User,
            2 => Category::Cpu,
            3 => Category::Memory,
            4 => Category::Disk,
            _ => unreachable!(),
        }
    }

    pub fn element(&self, theme: &theme::Theme, sort: &(Category, Sort)) -> Element<Message> {
        let cosmic = theme.cosmic();

        let header = widget::row::with_children(if sort.0 == self.clone() {
            match sort.1 {
                Sort::Ascending => vec![
                    widget::text::heading(self.name()).into(),
                    widget::icon::from_name("pan-up-symbolic").into(),
                ],
                Sort::Descending => vec![
                    widget::text::heading(self.name()).into(),
                    widget::icon::from_name("pan-down-symbolic").into(),
                ],
            }
        } else {
            vec![widget::text::heading(self.name()).into()]
        })
        .spacing(cosmic.space_xxs())
        .align_y(Vertical::Center);

        widget::button::custom(
            widget::container(header)
                .padding([cosmic.space_xxs(), cosmic.space_xs()])
                .width(self.width()),
        )
        .padding([0, 0])
        .on_press(Message::ProcessCategoryClick(self.index()))
        .class(cosmic::style::Button::HeaderBar)
        .into()
    }
}
