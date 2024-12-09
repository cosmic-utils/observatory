#[derive(Clone)]
pub struct Statistic {
    pub name: String,
    pub icon: String,
    pub percent: f32,
    pub hint: String,
}

impl Statistic {
    pub fn new(name: String, icon: String, percent: f32, hint: String) -> Self {
        Self {
            name,
            icon,
            percent,
            hint,
        }
    }
}
