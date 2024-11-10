#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    Refresh,
    ProcessTermActive,
    ProcessKillActive,
    ProcessClick(Option<sysinfo::Pid>),
}
