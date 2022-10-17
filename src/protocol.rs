use crate::data::*;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, from_value, to_value, to_writer, Value};
use std::{io::Write as _, str::FromStr};

// For full ser/de of commands
// ClientCommand can be serialized,
// ServerCommand can be deserialized
// But mutually they can suck one.

#[derive(Serialize, Deserialize, Debug)]
struct CommandDummy {
    command: String, // This was originally &'a str, but this caused serde to complain about Deserialize having insufficient lifetimes.
    #[serde(default)]
    data: Value,
}

// Use Serde to convert them to/from adjacent format
// These should later return a specific Result instead.
pub fn parse_command(command: &str) -> ServerCommand {
    from_value(
        to_value(
            if command.len() < 4 {
                // If the command has no body.
                CommandDummy {
                    command: command.to_owned(), 
                    data: Value::Null
                }
            } else {
                // Split the command into the JSON data body and the command head
                let (head, data) = command.split_at(4);
                CommandDummy {
                    command: head.trim().to_string(),
                    data: from_str(data).expect("Unable to parse data to Value"),
                }
            }
        )
        .expect("Unable to convert CommandDummy to Value"),
    )
    .expect("Unable to convert Value to ServerCommand") // Forgive me, for I have sinned.
}

pub fn prepare_command(command: &ClientCommand) -> String {
    let dummy_value: CommandDummy =
        from_value(to_value(command).expect("Unable to convert command to Value"))
            .expect("Unable to convert Value to CommandDummy");
    let mut command_buffer = Vec::with_capacity(256);
    write!(&mut command_buffer, "{} ", dummy_value.command).expect("Failed to write command head");
    to_writer(&mut command_buffer, &dummy_value.data).expect("Failed to write command data-body");
    String::from_utf8_lossy(&command_buffer).to_string()
}

// F-Chat commands follow a specific format:
// XXX {...}
// Being a 3 character command code, followed by JSON data
// This is fine.

#[derive(Serialize, PartialEq, Eq, Debug)]
#[serde(tag = "command", content = "data")]
pub enum ClientCommand {
    #[serde(rename = "ACB")]
    GlobalBan { character: Character },
    #[serde(rename = "AOP")]
    GlobalOp { character: Character },
    #[serde(rename = "CRC")]
    GlobalCreateChannel { name: String },
    #[serde(rename = "DOP")]
    GlobalDeop { character: Character },
    #[serde(rename = "AWC")]
    Alts { character: Character },
    #[serde(rename = "BRO")]
    Broadcast { message: String },
    #[serde(rename = "CBL")]
    Banlist { channel: Channel },
    #[serde(rename = "CBU")]
    Ban {
        channel: Channel,
        character: Character,
    },
    #[serde(rename = "COA")]
    Op {
        channel: Channel,
        character: Character,
    },
    #[serde(rename = "CCR")]
    CreateChannel { name: String },
    #[serde(rename = "CDS")]
    ChangeDescription {
        channel: Channel,
        description: String,
    },
    #[serde(rename = "CHA")]
    GlobalChannels,
    #[serde(rename = "CIU")]
    ChannelInviteUser {
        channel: Channel,
        character: Character,
    },
    #[serde(rename = "CKU")]
    Kick {
        channel: Channel,
        character: Character,
    },
    #[serde(rename = "COL")]
    Ops { channel: Channel },
    #[serde(rename = "COR")]
    Deop {
        channel: Channel,
        character: Character,
    },
    #[serde(rename = "CSO")]
    SetOwner {
        channel: Channel,
        character: Character,
    },
    #[serde(rename = "CTU")]
    Timeout {
        channel: Channel,
        character: Character,
    },
    #[serde(rename = "CUB")]
    Pardon {
        channel: Channel,
        character: Character,
    },
    #[serde(rename = "FKS")]
    Search {
        kinks: Vec<KinkId>,
        genders: Vec<Gender>,
        orientations: Vec<Orientation>,
        languages: Vec<Language>,
        furryprefs: Vec<FurryPreference>,
        roles: Vec<Role>,
    },
    #[serde(rename = "IDN")]
    Identify {
        method: IdentifyMethod,
        account: String,
        ticket: String,
        character: Character,
        #[serde(rename="cname")]
        client_name: String,
        #[serde(rename="cversion")]
        client_version: String,
    }, // method should always be "ticket"
    #[serde(rename = "IGN")]
    IgnoreList {
        action: IgnoreAction,
        character: Character,
    }, // This is terrible. Review later for constant field tagging.
    #[serde(rename = "JCH")]
    JoinChannel { channel: Channel },
    #[serde(rename = "KIC")]
    DeleteChannel { channel: Channel },
    #[serde(rename = "KIK")]
    GlobalKick { channel: Channel },
    #[serde(rename = "KIN")]
    Kinks { character: Character }, // Advised to use JSON endpoint
    #[serde(rename = "LCH")]
    LeaveChannel { channel: Channel },
    #[serde(rename = "LRP")]
    Ad { channel: Channel, message: String },
    #[serde(rename = "MSG")]
    Message { channel: Channel, message: String },
    #[serde(rename = "ORS")]
    Channels,
    #[serde(rename = "PIN")]
    Pong,
    #[serde(rename = "PRI")]
    PrivateMessage {
        recipient: Character,
        message: String,
    },
    #[serde(rename = "PRO")]
    ProfileTags { character: Character }, // Advised to use JSON endpoint
    #[serde(rename = "RLL")]
    Roll { channel: Channel, dice: String },
    #[serde(rename = "RLD")]
    Reload { save: String }, // ???
    #[serde(rename = "RMO")]
    ChannelMode { channel: Channel, mode: ChannelMode },
    #[serde(rename = "RST")]
    ChannelStatus {
        channel: Channel,
        status: ChannelStatus,
    },
    #[serde(rename = "RWD")]
    Reward { character: Character },
    #[serde(rename = "SFC")]
    Report {
        action: ReportAction,
        report: String,
        character: Character,
    }, // action is always 'report'
    #[serde(rename = "STA")]
    Status { status: Status, statusmsg: String },
    #[serde(rename = "TMO")]
    GlobalTimeout {
        character: Character,
        time: u32,
        reason: String,
    },
    #[serde(rename = "TPN")]
    Typing {
        character: Character,
        status: TypingStatus,
    },
    #[serde(rename = "UNB")]
    GlobalPardon { character: Character },
    #[serde(rename = "UPT")]
    Uptime,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(tag = "command", content = "data")]
pub enum ServerCommand {
    #[serde(rename = "ADL")]
    GlobalOps { ops: Vec<Character> },
    #[serde(rename = "AOP")]
    GlobalOpped { character: Character },
    #[serde(rename = "BRO")]
    Broadcast { message: String, character: Character },
    #[serde(rename = "CDS")]
    ChannelDescription {
        channel: Channel,
        description: String,
    },
    #[serde(rename = "CHA")]
    GlobalChannels { channels: Vec<Channel> },
    #[serde(rename = "CIU")]
    Invited {
        sender: Character,
        title: String,
        name: Channel,
    },
    #[serde(rename = "CBU")]
    Banned {
        operator: Character,
        channel: Channel,
        character: Character,
    },
    #[serde(rename = "CKU")]
    Kicked {
        operator: Character,
        channel: Channel,
        character: Character,
    },
    #[serde(rename = "COA")]
    Opped {
        character: Character,
        channel: Channel,
    },
    #[serde(rename = "COL")]
    Ops {
        channel: Channel,
        oplist: Vec<Character>,
    },
    #[serde(rename = "CON")]
    Connected { count: u32 },
    #[serde(rename = "COR")]
    Deopped {
        character: Character,
        channel: Channel,
    },
    #[serde(rename = "CSO")]
    SetOwner {
        character: Character,
        channel: Channel,
    },
    #[serde(rename = "CTU")]
    Timeout {
        channel: Channel,
        character: Character,
        length: u32,
        operator: Character,
    },
    #[serde(rename = "DOP")]
    GlobalDeopped { character: Character },
    #[serde(rename = "ERR")]
    Error { number: i32, message: String },
    #[serde(rename = "FKS")]
    Search {
        characters: Vec<Character>,
        kinks: Vec<KinkId>,
    },
    #[serde(rename = "FLN")]
    Offline { character: Character },
    #[serde(rename = "HLO")]
    Hello { message: String },
    #[serde(rename = "ICH")]
    ChannelData {
        users: Vec<CharacterIdentity>,
        channel: Channel,
        mode: ChannelMode,
    },
    #[serde(rename = "IDN")]
    IdentifySuccess { character: Character },
    #[serde(rename = "JCH")]
    JoinedChannel {
        channel: Channel,
        character: CharacterIdentity,
        title: String,
    },
    #[serde(rename = "KID")]
    Kinks {
        #[serde(rename = "type")]
        response_type: KinkResponsePart,
        message: String,
        key: Vec<u32>,
        value: Vec<u32>,
    },
    #[serde(rename = "LCH")]
    LeftChannel {
        channel: Channel,
        character: Character,
    },
    #[serde(rename = "LIS")]
    ListOnline { characters: Vec<FlatCharacterData> },
    #[serde(rename = "NLN")]
    NewConnection {
        status: Status,
        gender: Gender,
        identity: Character,
    },
    #[serde(rename = "IGN")]
    Ignore {
        action: IgnoreAction,
        #[serde(default)]
        characters: Vec<Character>,
        #[serde(default)]
        character: Character,
    }, // Thanks, F-List. 'Characters' only when init/list
    #[serde(rename = "FRL")]
    Friends { characters: Vec<Character> },
    #[serde(rename = "ORS")]
    Channels { channels: Vec<ChannelInfo> },
    #[serde(rename = "PIN")]
    Ping,
    #[serde(rename = "PRD")]
    ProfileData {
        #[serde(rename = "type")]
        response_type: ProfileDataPart,
        #[serde(default)]
        message: String,
        #[serde(default)]
        key: String,
        #[serde(default)]
        value: String,
    },
    #[serde(rename = "PRI")]
    PrivateMessage {
        character: Character,
        message: String,
    },
    #[serde(rename = "MSG")]
    Message {
        character: Character,
        message: String,
        channel: Channel,
    },
    #[serde(rename = "LRP")]
    Ad {
        character: Character,
        message: String,
        channel: Channel,
    },
    #[serde(rename = "RLL")]
    Roll {
        channel: Channel,
        results: u32,
        #[serde(rename = "type")]
        response_type: String,
        rolls: Vec<String>,
        character: Character,
        endresult: u32,
        message: String,
    },
    #[serde(rename = "RMO")]
    ChannelMode { mode: ChannelMode, channel: Channel },
    #[serde(rename = "RTB")]
    BridgeEvent {
        #[serde(rename = "type")]
        response_type: String,
        character: Character,
    },
    #[serde(rename = "SFC")]
    Report {
        action: String,
        moderator: Character,
        character: Character,
        timestamp: String,
        callid: u32,
        report: String,
        logid: u32,
    }, // May need to flatten an enum in here instead.
    #[serde(rename = "STA")]
    Status {
        status: Status,
        character: Character,
        statusmsg: String,
    },
    #[serde(rename = "SYS")]
    SystemMessage { message: String, channel: Channel }, // Catch-all response for many things. Fuck this universe.
    #[serde(rename = "TPN")]
    Typing {
        character: Character,
        status: TypingStatus,
    },
    #[serde(rename = "UPT")]
    Uptime {
        time: u64,
        starttime: u64,
        startstring: String,
        accepted: u64,
        channels: u32,
        users: u32,
        maxusers: u32,
    },
    #[serde(rename = "VAR")]
    Variable(Variable), // Could be int, float, [string]; I hate it. Use an adjacently tagged enum.
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "variable", content = "value")]
#[serde(rename_all = "snake_case")]
pub enum Variable {
    ChatMax(u32),
    PrivMax(u32),
    #[serde(rename = "lfrp_max")]
    AdMax(u32),
    #[serde(rename = "cds_max")]
    CDSMax(u32), // What is this?
    #[serde(rename = "lfrp_flood")]
    AdCooldown(f32),
    #[serde(rename = "msg_flood")]
    ChatCooldown(f32),
    #[serde(rename = "sta_flood")]
    StatusCooldown(f32),
    Permissions(String),
    IconBlacklist(Vec<Channel>),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct CharacterIdentity {
    pub identity: Character,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum KinkResponsePart {
    Start,
    Custom,
    End,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct FlatCharacterData(pub Character, pub Gender, pub Status, pub String); // Last part is status message

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct ChannelInfo {
    pub name: Channel,
    pub characters: u32,
    pub title: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ProfileDataPart {
    Start,
    End,
    Info,
    Select,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ReportAction {
    Report,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum IdentifyMethod {
    Ticket,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum IgnoreAction {
    Add,
    Delete,
    Notify,
    List,
    Init,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
#[serde(into="KinkIdExpanded")]
#[serde(try_from="KinkIdExpanded")]
pub struct KinkId(pub u32);
impl From<KinkId> for KinkIdExpanded {
    fn from(val: KinkId) -> Self {
        KinkIdExpanded::String(val.0.to_string())
    }
}
impl TryFrom<KinkIdExpanded> for KinkId {
    type Error = <u32 as FromStr>::Err;
    fn try_from(other: KinkIdExpanded) -> Result<KinkId, Self::Error> {
        match other {
            KinkIdExpanded::String(val) => val.parse().map(KinkId),
            KinkIdExpanded::Number(val) => Ok(KinkId(val))
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(untagged)]
enum KinkIdExpanded {
    String(String),
    Number(u32)
}

#[derive(Debug, num_enum::FromPrimitive)]
#[repr(i32)]
pub enum ProtocolError {
    Success = 0, // Not an error.
    SytaxError = 1,
    FullServer = 2,
    Unauthenticated = 3,
    AuthenticationFailed = 4,
    MessageCooldown = 5,
    NoSuchCharacter = 6,
    ProfileCooldown = 7,
    UnknownCommand = 8,
    Banned = 9,
    AdminRequired = 10,
    AlreadyIdentified = 11,
    KinkCooldown = 13,
    MessageTooLong = 15,
    AlreadyModerator = 16,
    NotAModerator = 17,
    NoResults = 18,
    ModeratorRequired = 19,
    Ignored = 20,
    InvalidTarget = 21,
    NoSuchChannel = 26,
    AlreadyInChannel = 28,
    TooManySessions = 30,
    AnotherConnection = 31,
    AlreadyBanned = 32,
    InvalidAuthentication = 33, // Hopefully this never crops up.
    RollError = 36,
    InvalidTimeoutDuration = 38,
    TimeOut = 39,
    Kick = 40,
    AlreadyChannelBanned = 41,
    NotChannelBanned = 42,
    ChannelInviteRequired = 44,
    ChannelJoinRequired = 45,
    ChannelInviteForbidden = 47,
    ChannelBanned = 48,
    CharacterNotInChannel = 49,
    SearchCooldown = 50,
    ReportCooldown = 54,
    AdCooldown = 56,
    MessageOnly = 59,
    AdsOnly = 60,
    TooManySearchTerms = 61,
    NoFreeSlots = 62,
    IgnoreListTooLong = 64,
    ChannelTitleTooLong = 67,
    TooManySearchResults = 72,

    InternalError = -1,
    CommandError = -2,
    Unimplemented = -3,
    LoginTimeOut = -4,
    UnknownError = -5,
    FrontpageDice = -10,

    #[num_enum(default)]
    Other = 9999
}

impl ProtocolError {
    /// If an error `is_fatal` then the client should not attempt reconnection if:
    /// - It is disconnected
    /// - This error was the most recent before disconnection
    pub fn is_fatal(&self) -> bool {
        /*
        2: The server is full and not able to accept additional connections at this time.
        9: The user is banned.
        30: The user is already connecting from too many clients at this time.
        31: The character is logging in from another location and this connection is being terminated.
        33: Invalid authentication method used.
        39: The user is being timed out from the server.
        40: The user is being kicked from the server.
        */
        matches!(self, Self::FullServer | Self::Banned | Self::TooManySessions | Self::AnotherConnection | Self::InvalidAuthentication | Self::TimeOut | Self::Kick | Self::InternalError)
    }

    /// The errors themselves do not literally contain messages, but they are often transmitted with messages.
    /// If an error object has a message, it means that the associated message is important and contains information about
    /// the specific error; the error is not statically associated with the ID, and has variables interpolated.
    pub fn has_message(&self) -> bool {
        matches!(self, Self::Ignored | Self::TimeOut)
    }
}