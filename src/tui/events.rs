use crate::network::protocol::Channel;
use crate::tui::Focus;
use crate::tui::framework::FromLog;
use crate::tui::logs::LogEntry;

#[derive(Debug)]
pub enum TuiEvent {
    Log(LogEntry),
    Exit,
    ChannelUp,
    ChannelDown,
    FocusChange(Focus),
    InputRight,
    InputRightTab,
    InputLeft,
    InputLeftTab,
    InputChar(char),
    InputDelete,
    InputEnter,
    ToggleLogs,
    LoggedIn,
    HealthCheck,
    SetUserName(String),
    Disconnected,
    Channels(Vec<Channel>),
    ChannelIDs(Vec<u64>),
    ScrollUp,
    ScrollDown,
}

impl FromLog for TuiEvent {
    fn from_log(log: LogEntry) -> Self {
        TuiEvent::Log(log)
    }
}
