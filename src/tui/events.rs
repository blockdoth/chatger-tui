use std::net::SocketAddr;

use crate::network::protocol::UserStatus;
use crate::network::protocol::server::{Channel, HistoryMessage, UserData};
use crate::tui::chat::MediaMessage;
use crate::tui::framework::FromLog;
use crate::tui::logs::LogEntry;
use crate::tui::screens::chat::state::ChatFocus;
use crate::tui::screens::login::state::LoginFocus;

pub type UserId = u64;
pub type ChannelId = u64;
pub type MessageId = u64;
pub type MediaId = u64;
pub type ProfilePicId = u64;
pub type IconId = u64;

#[derive(Debug)]
pub enum TuiEvent {
    Log(LogEntry),
    Exit,
    ChannelUp,
    ChannelDown,
    ChatFocusChange(ChatFocus),
    LoginFocusChange(LoginFocus),
    InputRight,
    InputRightTab,
    InputLeft,
    InputLeftTab,
    InputChar(char),
    InputDelete,
    InputEnter,
    ToggleLogs,
    LoginSuccess(UserId),
    Login,
    LoginFail(String),
    HealthCheck,
    SetUserNamePassword(String, String),
    Disconnected,
    Channels(Vec<Channel>),
    ChannelIDs(Vec<ChannelId>),
    ScrollUp,
    ScrollDown,
    ConnectAndLogin(SocketAddr, String, String),
    UserStatusesUpdate(Vec<(UserId, UserStatus)>),
    UserStatusUpdate(UserId, UserStatus),
    Users(Vec<UserData>),
    HistoryUpdate(Vec<HistoryMessage>),
    MessageSendAck(MessageId),
    MessageMediaAck(MediaId),
    Media(MediaMessage),
    Typing(ChannelId, UserId, bool),
}

impl FromLog for TuiEvent {
    fn from_log(log: LogEntry) -> Self {
        TuiEvent::Log(log)
    }
}
