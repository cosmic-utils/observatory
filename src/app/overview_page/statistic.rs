
pub struct Statistic {
    pub name: String,
    pub percent: f32,
}

impl Statistic {
    pub fn new(name: String, percent: f32) -> Self {
        Self { name, percent }
    }
}