use crate::fl;

#[derive(Clone, Debug, PartialEq)]
pub enum ContextPage {
    About,
    Settings,
    ProcInfo,
}

impl ContextPage {
    pub fn title(&self) -> String {
        match self {
            ContextPage::About => fl!("about"),
            ContextPage::Settings => fl!("settings"),
            ContextPage::ProcInfo => fl!("proc-info"),
        }
    }
}
