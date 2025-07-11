use std::net::SocketAddr;

use crate::network::protocol::{Channel, HistoryMessage, UserData, UserStatus};
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
    SetUserNamePassword(String, String),
    Disconnected,
    Channels(Vec<Channel>),
    ChannelIDs(Vec<u64>),
    ScrollUp,
    ScrollDown,
    ConnectAndLogin(SocketAddr, String, String),
    UserStatusesUpdate(Vec<(u64, UserStatus)>),
    Users(Vec<UserData>),
    HistoryUpdate(Vec<HistoryMessage>),
    MessageSendAck(u64),
}

impl FromLog for TuiEvent {
    fn from_log(log: LogEntry) -> Self {
        TuiEvent::Log(log)
    }
}
