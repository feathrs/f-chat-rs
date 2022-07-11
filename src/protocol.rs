use crate::data::*;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, from_value, to_value, to_writer, Value};
use std::io::Write as _;

// For full ser/de of commands
// ClientCommand can be serialized,
// ServerCommand can be deserialized
// But mutually they can suck one.

#[derive(Serialize, Deserialize)]
struct CommandDummy {
    command: String, // This was originally &'a str, but this caused serde to complain about Deserialize having insufficient lifetimes.
    data: Value,
}

// Use Serde to convert them to/from adjacent format
pub fn parse_command(command: &str) -> ServerCommand {
    // Split the command into the JSON data body and the command head
    let (head, data) = command.split_at(4);
    from_value(
        to_value(CommandDummy {
            command: head.trim().to_string(),
            data: from_str(data).expect("Unable to parse data to Value"),
        })
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

#[derive(Serialize)]
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
        kinks: Vec<u32>,
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
        cname: String,
        cversion: String,
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

#[derive(Deserialize)]
#[serde(tag = "command", content = "data")]
pub enum ServerCommand {
    #[serde(rename = "ADL")]
    GlobalOps { ops: Vec<Character> },
    #[serde(rename = "AOP")]
    GlobalOpped { character: Character },
    #[serde(rename = "BRO")]
    Broadcast { message: String },
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
    Error { number: u32, message: String },
    #[serde(rename = "FKS")]
    Search {
        characters: Vec<Character>,
        kinks: Vec<u32>,
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
    ListOnline { characters: Vec<CharacterData> },
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
        characters: Vec<String>,
        #[serde(default)]
        character: Character,
    }, // Thanks, F-List. 'Characters' only when init/list
    #[serde(rename = "FRL")]
    Friends { characters: String },
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
    Variable {
        variable: String,
        value: serde_json::Value,
    }, // Could be int, float, [string]; I hate it. Use an adjacently tagged enum.
}