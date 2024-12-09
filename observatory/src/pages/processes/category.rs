use cosmic::iced::Length;

pub struct CategoryList(pub Vec<Category>);

#[derive(PartialEq, Clone)]
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
    Pid,
    Cpu,
    Gpu,
    Memory,
    Disk,
}

impl CategoryList {
    pub fn new() -> Self {
        CategoryList(
            [
                Category::Name,
                Category::Pid,
                Category::Cpu,
                Category::Gpu,
                Category::Memory,
                Category::Disk,
            ]
            .into(),
        )
    }
}

impl Category {
    pub fn name(&self) -> String {
        match self {
            Category::Name => "Name",
            Category::Pid => "Pid",
            Category::Cpu => "CPU",
            Category::Gpu => "GPU",
            Category::Memory => "Memory",
            Category::Disk => "Disk",
        }
        .into()
    }

    pub fn width(&self) -> Length {
        match self {
            Category::Name => Length::Fixed(300.),
            Category::Pid => Length::Fixed(66.),
            Category::Cpu => Length::Fixed(70.),
            Category::Gpu => Length::Fixed(70.),
            Category::Memory => Length::Fixed(100.),
            Category::Disk => Length::Fixed(120.),
        }
    }

    pub fn index(&self) -> u8 {
        match self {
            Category::Name => 0,
            Category::Pid => 1,
            Category::Cpu => 2,
            Category::Gpu => 3,
            Category::Memory => 4,
            Category::Disk => 5,
        }
    }

    pub fn from_index(index: u8) -> Category {
        match index {
            0 => Category::Name,
            1 => Category::Pid,
            2 => Category::Cpu,
            3 => Category::Gpu,
            4 => Category::Memory,
            5 => Category::Disk,
            _ => unreachable!(),
        }
    }
}
