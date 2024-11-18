pub struct Statistic {
    pub name: String,
    pub icon: &'static str,
    pub percent: f32,
}

impl Statistic {
    pub fn new(name: String, icon: &'static str, percent: f32) -> Self {
        Self {
            name,
            icon,
            percent,
        }
    }
}
