use serde::{Serialize, Deserialize};

// F-Chat commands follow a specific format:
// XXX {...}
// Being a 3 character command code, followed by JSON data
// This is fine.

#[derive(Serialize)]
enum ClientCommand {
    GlobalBan {character: String},
    GlobalOp {character: String},
    GlobalCreateChannel {name: String},
    GlobalDeop {character: String},
    Alts {character: String},
    Broadcast {message: String},
    Banlist {channel: String},
    Ban {channel: String, character: String},
    Op {channel: String, character: String},
    CreateChannel {name: String},
    ChangeDescription {channel: String, description: String},
    GlobalChannels,
    ChannelInviteUser {channel: String, character: String},
    Kick {channel: String, character: String},
    Ops {channel: String},
    Deop {channel: String, character: String},
    SetOwner {channel: String, character: String},
    Timeout {channel: String, character: String},
    Pardon {channel: String, character: String},
    Search {kinks: Vec<u32>, genders: Vec<Gender>, orientations: Vec<Orientation>, languages: Vec<Language>, furryprefs: Vec<FurryPreference>, roles: Vec<Role>},
    Identify {method: IdentifyMethod, account: String, ticket: String, character: String, cname: String, cversion: String}, // method should always be "ticket"
    IgnoreList {action: IgnoreAction, character: String}, // This is terrible. Review later for constant field tagging.
    JoinChannel {channel: String},
    DeleteChannel {channel: String},
    GlobalKick {channel: String},
    Kinks {character: String}, // Advised to use JSON endpoint
    LeaveChannel {channel: String},
    Ad {channel: String, message: String},
    Message {channel: String, message: String},
    Channels,
    Pong,
    PrivateMessage {recipient: String, message: String},
    ProfileTags {character: String}, // Advised to use JSON endpoint
    Roll {channel: String, dice: String},
    Reload {save: String}, // ???
    ChannelMode {channel: String, mode: ChannelMode},
    ChannelStatus {channel: String, status: ChannelStatus},
    Reward {character: String},
    Report {action: ReportAction, report: String, character: String}, // action is always 'report'
    Status {status: Status, statusmsg: String},
    GlobalTimeout {character: String, time: u32, reason: String},
    Typing {character: String, status: TypingStatus},
    GlobalPardon {character: String},
    Uptime
}

impl ClientCommand {
    fn command_code(&self) -> &'static str {
        match self {
            ClientCommand::GlobalBan {..} => "ACB",
            ClientCommand::GlobalOp {..} => "AOP",
            ClientCommand::GlobalCreateChannel {..} => "CRC",
            ClientCommand::GlobalDeop {..} => "DOP",
            ClientCommand::Alts {..} => "AWC",
            ClientCommand::Broadcast {..} => "BRO",
            ClientCommand::Banlist {..} => "CBL",
            ClientCommand::Ban {..} => "CBU",
            ClientCommand::Op {..} => "COA",
            ClientCommand::CreateChannel {..} => "CCR",
            ClientCommand::ChangeDescription {..} => "CDS",
            ClientCommand::GlobalChannels => "CHA",
            ClientCommand::ChannelInviteUser {..} => "CIU",
            ClientCommand::Kick {..} => "CKU",
            ClientCommand::Ops {..} => "COL",
            ClientCommand::Deop {..} => "COR",
            ClientCommand::SetOwner {..} => "CSO",
            ClientCommand::Timeout {..} => "CTU",
            ClientCommand::Pardon {..} => "CUB",
            ClientCommand::Search {..} => "FKS",
            ClientCommand::Identify {..} => "IDN",
            ClientCommand::IgnoreList {..} => "IGN",
            ClientCommand::JoinChannel {..} => "JCH",
            ClientCommand::DeleteChannel {..} => "KIC",
            ClientCommand::GlobalKick {..} => "KIK",
            ClientCommand::Kinks {..} => "KIN",
            ClientCommand::LeaveChannel {..} => "LCH",
            ClientCommand::Ad {..} => "LRP",
            ClientCommand::Message {..} => "MSG",
            ClientCommand::Channels => "ORS",
            ClientCommand::Pong => "PIN",
            ClientCommand::PrivateMessage {..} => "PRI",
            ClientCommand::ProfileTags {..} => "PRO",
            ClientCommand::Roll {..} => "RLL",
            ClientCommand::Reload {..} => "RLD",
            ClientCommand::ChannelMode {..} => "RMO",
            ClientCommand::ChannelStatus {..} => "RST",
            ClientCommand::Reward {..} => "RWD",
            ClientCommand::Report {..} => "SFC",
            ClientCommand::Status {..} => "STA",
            ClientCommand::GlobalTimeout {..} => "TMO",
            ClientCommand::Typing {..} => "TPN",
            ClientCommand::GlobalPardon {..} => "UNB",
            ClientCommand::Uptime => "UPT",
        }
    }
}

#[derive(Deserialize)]
enum ServerCommand {
    GlobalOps {ops: Vec<String>},
    GlobalOpped {character: String},
    Broadcast {message: String},
    ChannelDescription {channel: String, description: String},
    GlobalChannels {channels: Vec<String>},
    Invited {sender: String, title: String, name: String},
    Banned {operator: String, channel: String, character: String},
    Kicked {operator: String, channel: String, character: String},
    Opped {character: String, channel: String},
    Ops {channel: String, oplist: Vec<String>},
    Connected {count: u32},
    Deopped {character: String, channel: String},
    SetOwner {character: String, channel: String},
    Timeout {channel: String, character: String, length: u32, operator: String},
    GlobalDeopped {character: String},
    Error {number: u32, message: String},
    Search {characters: Vec<String>, kinks: Vec<u32>},
    Offline {character: String},
    Hello {message: String},
    ChannelData {users: Vec<CharacterIdentity>, channel: String, mode: ChannelMode},
    IdentifySuccess {character: String},
    JoinedChannel {channel: String, character: CharacterIdentity, title: String},
    Kinks {#[serde(rename="type")] response_type: KinkResponsePart, message: String, key: Vec<u32>, value: Vec<u32>},
    LeftChannel {channel: String, character: String},
    ListOnline {characters: Vec<CharacterData>},
    NewConnection {status: Status, gender: Gender, identity: String},
    Ignore {action: IgnoreAction, #[serde(default)] characters: Vec<String>, #[serde(default)] character: String}, // Thanks, F-List. 'Characters' only when init/list
    Friends {characters: String},
    Channels {channels: Vec<ChannelInfo>},
    Ping,
    ProfileData {#[serde(rename="type")] response_type: ProfileDataPart, #[serde(default)] message: String, #[serde(default)] key: String, #[serde(default)] value: String},
    PrivateMessage {character: String, message: String},
    Message {character: String, message: String, channel: String},
    Ad {character: String, message: String, channel: String},
    Roll {channel: String, results: u32, #[serde(rename="type")] response_type: String, rolls: Vec<String>, character: String, endresult: u32, message: String},
    ChannelMode {mode: ChannelMode, channel: String},
    BridgeEvent {#[serde(rename="type")] response_type: String, character: String},
    Report {action: String, moderator: String, character: String, timestamp: String, callid: u32, report: String, logid: u32}, // May need to flatten an enum in here instead. 
    Status {status: Status, character: String, statusmsg: String},
    SystemMessage {message: String, channel: String}, // Catch-all response for many things. Fuck this universe.
    Typing {character: String, status: TypingStatus},
    Uptime {time: u64, starttime: u64, startstring: String, accepted: u64, channels: u32, users: u32, maxusers: u32},
    Variable {variable: String, value: serde_json::Value}, // Could be int, float, [string]; I hate it. Use an adjacently tagged enum.
}

#[derive(Serialize, Deserialize)]
enum Gender {
    Male, Female, Transgender, Herm, Shemale, MaleHerm, CBoy, None
}

#[derive(Serialize, Deserialize)]
enum Orientation {
    Straight, Gay, Bisexual, Asexual, Unsure, BiMalePref, BiFemalePref, Pansexual, Bicurious
}

#[derive(Serialize, Deserialize)]
enum Language {
    Dutch, English, French, Spanish, German, Russian, Chinese, Japanese, Portuguese, Korean, Arabic, Italian, Swedish, Other
}

#[derive(Serialize, Deserialize)]
enum FurryPreference {
    HumanOnly, HumanPref, Both, FurryPref, FurryOnly
}

#[derive(Serialize, Deserialize)]
enum Role {
    AlwaysDom, UsuallyDom, Switch, UsuallySub, AlwaysSub, None
}

#[derive(Serialize, Deserialize)]
enum IdentifyMethod {
    Ticket
}

#[derive(Serialize, Deserialize)]
enum IgnoreAction {
    Add, Delete, Notify, List, Init
}

#[derive(Serialize, Deserialize)]
enum ChannelMode {
    ChatOnly, AdsOnly, Both
}

#[derive(Serialize, Deserialize)]
enum ChannelStatus {
    Public, Private
}

#[derive(Serialize, Deserialize)]
enum ReportAction {
    Report
}

#[derive(Serialize, Deserialize)]
enum Status {
    Online, Looking, Busy, Dnd, Idle, Away, Crown
    // If you try to set crown, you will die.
}

#[derive(Serialize, Deserialize)]
enum TypingStatus {
    Clear, Paused, Typing
}

#[derive(Serialize, Deserialize)]
struct CharacterIdentity {
    identity: String
}

#[derive(Serialize, Deserialize)]
enum KinkResponsePart {
    Start, Custom, End
}

#[derive(Serialize, Deserialize)]
struct CharacterData(String, Gender, Status, String);

#[derive(Serialize, Deserialize)]
struct ChannelInfo {
    name: String,
    characters: u32, 
    title: String
}

#[derive(Serialize, Deserialize)]
enum ProfileDataPart {
    Start, End, Info, Select
}