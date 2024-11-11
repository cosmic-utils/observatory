#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    Refresh,
    KeyPressed(cosmic::iced_core::keyboard::Key),

    ProcessTermActive,
    ProcessKillActive,
    ProcessClick(Option<sysinfo::Pid>),
}
