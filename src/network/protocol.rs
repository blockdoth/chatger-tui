#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum PacketVersion {
    V1 = 0x01,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Status {
    Success = 0x00,
    Failed = 0x01,
    Notification = 0x02, // Only used for HISTORY
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum MediaType {
    Raw = 0x00,
    Text = 0x01,
    Audio = 0x02,
    Image = 0x03,
    Video = 0x04,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum UserStatus {
    Offline = 0x00,
    Online = 0x01,
    Idle = 0x02,
    DoNotDisturb = 0x03,
}

#[derive(Debug)]
pub struct PacketHeader {
    pub magic_number: [u8; 4], // "CHTG"
    pub version: PacketVersion,
    pub user_flag_and_id: u8, // [is_user|1][packet_id|7]
    pub length: u32,          // length of content in bytes
}

impl PacketHeader {
    pub fn is_user(&self) -> bool {
        self.user_flag_and_id & 0b1000_0000 != 0
    }

    pub fn packet_id(&self) -> u8 {
        self.user_flag_and_id & 0b0111_1111
    }
}

#[derive(Debug)]
pub enum PacketContent {
    HealthCheck(HealthCheckPacket),
    Login(LoginPacket),
    LoginResponse(LoginResponsePacket),
    SendMessage(SendMessagePacket),
    SendMessageAck(SendMessageAckPacket),
    SendMedia(SendMediaPacket),
    SendMediaAck(SendMediaAckPacket),
    GetChannelsList,
    ChannelsList(ChannelsListPacket),
    GetChannels(GetChannelsPacket),
    Channels(ChannelsPacket),
    GetHistory(GetHistoryPacket),
    History(HistoryPacket),
    GetUsersList,
    UsersList(UsersListPacket),
    GetUsers(GetUsersPacket),
    Users(UsersPacket),
    GetMedia(GetMediaPacket),
    Media(MediaPacket),
    Typing(TypingPacket),
    UserTyping(UserTypingPacket),
    Status(StatusPacket),
    UserStatus(UserStatusPacket),
    UserConfigSet(UserConfigSetPacket), // TODO
    UserConfigAck(UserConfigAckPacket), // TODO
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum HealthKind {
    Ping = 0x00,
    Pong = 0x01,
}

#[derive(Debug)]
pub struct HealthCheckPacket {
    pub kind: HealthKind,
}

#[derive(Debug)]
pub struct LoginPacket {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct LoginResponsePacket {
    pub status: Status,
    pub error_message: String,
}

#[derive(Debug)]
pub struct SendMessagePacket {
    pub channel_id: u64,
    pub reply_id: u64,
    pub media_ids: Vec<u64>,
    pub message_text: String,
}

#[derive(Debug)]
pub struct SendMessageAckPacket {
    pub status: Status,
    pub message_id: u64,
    pub error_message: String,
}

#[derive(Debug)]
pub struct SendMediaPacket {
    pub filename: String,
    pub media_type: MediaType,
    pub media_data: Vec<u8>,
}

#[derive(Debug)]
pub struct SendMediaAckPacket {
    pub status: Status,
    pub media_id: u64,
    pub error_message: String,
}

#[derive(Debug)]
pub struct GetChannelsPacket {
    pub channel_ids: Vec<u64>,
}

#[derive(Debug)]
pub struct ChannelsListPacket {
    pub status: Status,
    pub channel_ids: Vec<u64>,
    pub error_message: String,
}

#[derive(Debug)]
pub struct ChannelInfo {
    pub channel_id: u64,
    pub name: String,
    pub icon_id: u64,
}

#[derive(Debug)]
pub struct ChannelsPacket {
    pub status: Status,
    pub channels: Vec<ChannelInfo>,
    pub error_message: String,
}

#[derive(Debug)]
pub enum Anchor {
    Timestamp(u64), // MSB = 0
    MessageId(u64), // MSB = 1
}

#[derive(Debug)]
pub struct GetHistoryPacket {
    pub channel_id: u64,
    pub anchor: Anchor,
    pub num_messages_back: i8,
}

#[derive(Debug)]
pub struct HistoryMessage {
    pub message_id: u64,
    pub sent_timestamp: u64,
    pub user_id: u64,
    pub channel_id: u64,
    pub reply_id: u64,
    pub message_text: String,
    pub media_ids: Vec<u64>,
}

#[derive(Debug)]
pub struct HistoryPacket {
    pub status: Status,
    pub messages: Vec<HistoryMessage>,
    pub error_message: String,
}

#[derive(Debug)]
pub struct GetUsersPacket {
    pub user_ids: Vec<u64>,
}

#[derive(Debug)]
pub struct UsersListPacket {
    pub status: Status,
    pub users: Vec<(u64, UserStatus)>,
    pub error_message: String,
}

#[derive(Debug)]
pub struct UserData {
    pub user_id: u64,
    pub status: UserStatus,
    pub username: String,
    pub pfp_id: u64,
    pub bio: String,
}

#[derive(Debug)]
pub struct UsersPacket {
    pub status: Status,
    pub users: Vec<UserData>,
    pub error_message: String,
}

#[derive(Debug)]
pub struct GetMediaPacket {
    pub media_id: u64,
}

#[derive(Debug)]
pub struct MediaPacket {
    pub status: Status,
    pub filename: String,
    pub media_type: MediaType,
    pub media_data: Vec<u8>,
    pub error_message: String,
}

#[derive(Debug)]
pub struct TypingPacket {
    pub is_typing: bool,
    pub channel_id: u64,
}

#[derive(Debug)]
pub struct UserTypingPacket {
    pub is_typing: bool,
    pub user_id: u64,
    pub channel_id: u64,
}

#[derive(Debug)]
pub struct StatusPacket {
    pub status: UserStatus,
}

#[derive(Debug)]
pub struct UserStatusPacket {
    pub status: UserStatus,
    pub user_id: u64,
}

#[derive(Debug)]
pub struct UserConfigSetPacket {
    // TODO: Define fields
}

#[derive(Debug)]
pub struct UserConfigAckPacket {
    // TODO: Define fields
}
