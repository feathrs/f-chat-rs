use serde::{Serialize, Deserialize};

// F-Chat commands follow a specific format:
// XXX {...}
// Being a 3 character command code, followed by JSON data
// This is fine.

#[derive(Serialize)]
#[serde(tag = "command", content = "data")]
enum ClientCommand {
    #[serde(rename="ACB")] GlobalBan {character: String},
    #[serde(rename="AOP")] GlobalOp {character: String},
    #[serde(rename="CRC")] GlobalCreateChannel {name: String},
    #[serde(rename="DOP")] GlobalDeop {character: String},
    #[serde(rename="AWC")] Alts {character: String},
    #[serde(rename="BRO")] Broadcast {message: String},
    #[serde(rename="CBL")] Banlist {channel: String},
    #[serde(rename="CBU")] Ban {channel: String, character: String},
    #[serde(rename="COA")] Op {channel: String, character: String},
    #[serde(rename="CCR")] CreateChannel {name: String},
    #[serde(rename="CDS")] ChangeDescription {channel: String, description: String},
    #[serde(rename="CHA")] GlobalChannels,
    #[serde(rename="CIU")] ChannelInviteUser {channel: String, character: String},
    #[serde(rename="CKU")] Kick {channel: String, character: String},
    #[serde(rename="COL")] Ops {channel: String},
    #[serde(rename="COR")] Deop {channel: String, character: String},
    #[serde(rename="CSO")] SetOwner {channel: String, character: String},
    #[serde(rename="CTU")] Timeout {channel: String, character: String},
    #[serde(rename="CUB")] Pardon {channel: String, character: String},
    #[serde(rename="FKS")] Search {kinks: Vec<u32>, genders: Vec<Gender>, orientations: Vec<Orientation>, languages: Vec<Language>, furryprefs: Vec<FurryPreference>, roles: Vec<Role>},
    #[serde(rename="IDN")] Identify {method: IdentifyMethod, account: String, ticket: String, character: String, cname: String, cversion: String}, // method should always be "ticket"
    #[serde(rename="IGN")] IgnoreList {action: IgnoreAction, character: String}, // This is terrible. Review later for constant field tagging.
    #[serde(rename="JCH")] JoinChannel {channel: String},
    #[serde(rename="KIC")] DeleteChannel {channel: String},
    #[serde(rename="KIK")] GlobalKick {channel: String},
    #[serde(rename="KIN")] Kinks {character: String}, // Advised to use JSON endpoint
    #[serde(rename="LCH")] LeaveChannel {channel: String},
    #[serde(rename="LRP")] Ad {channel: String, message: String},
    #[serde(rename="MSG")] Message {channel: String, message: String},
    #[serde(rename="ORS")] Channels,
    #[serde(rename="PIN")] Pong,
    #[serde(rename="PRI")] PrivateMessage {recipient: String, message: String},
    #[serde(rename="PRO")] ProfileTags {character: String}, // Advised to use JSON endpoint
    #[serde(rename="RLL")] Roll {channel: String, dice: String},
    #[serde(rename="RLD")] Reload {save: String}, // ???
    #[serde(rename="RMO")] ChannelMode {channel: String, mode: ChannelMode},
    #[serde(rename="RST")] ChannelStatus {channel: String, status: ChannelStatus},
    #[serde(rename="RWD")] Reward {character: String},
    #[serde(rename="SFC")] Report {action: ReportAction, report: String, character: String}, // action is always 'report'
    #[serde(rename="STA")] Status {status: Status, statusmsg: String},
    #[serde(rename="TMO")] GlobalTimeout {character: String, time: u32, reason: String},
    #[serde(rename="TPN")] Typing {character: String, status: TypingStatus},
    #[serde(rename="UNB")] GlobalPardon {character: String},
    #[serde(rename="UPT")] Uptime
}

#[derive(Deserialize)]
#[serde(tag = "command", content = "data")]
enum ServerCommand {
    #[serde(rename="ADL")] GlobalOps {ops: Vec<String>},
    #[serde(rename="AOP")] GlobalOpped {character: String},
    #[serde(rename="BRO")] Broadcast {message: String},
    #[serde(rename="CDS")] ChannelDescription {channel: String, description: String},
    #[serde(rename="CHA")] GlobalChannels {channels: Vec<String>},
    #[serde(rename="CIU")] Invited {sender: String, title: String, name: String},
    #[serde(rename="CBU")] Banned {operator: String, channel: String, character: String},
    #[serde(rename="CKU")] Kicked {operator: String, channel: String, character: String},
    #[serde(rename="COA")] Opped {character: String, channel: String},
    #[serde(rename="COL")] Ops {channel: String, oplist: Vec<String>},
    #[serde(rename="CON")] Connected {count: u32},
    #[serde(rename="COR")] Deopped {character: String, channel: String},
    #[serde(rename="CSO")] SetOwner {character: String, channel: String},
    #[serde(rename="CTU")] Timeout {channel: String, character: String, length: u32, operator: String},
    #[serde(rename="DOP")] GlobalDeopped {character: String},
    #[serde(rename="ERR")] Error {number: u32, message: String},
    #[serde(rename="FKS")] Search {characters: Vec<String>, kinks: Vec<u32>},
    #[serde(rename="FLN")] Offline {character: String},
    #[serde(rename="HLO")] Hello {message: String},
    #[serde(rename="ICH")] ChannelData {users: Vec<CharacterIdentity>, channel: String, mode: ChannelMode},
    #[serde(rename="IDN")] IdentifySuccess {character: String},
    #[serde(rename="JCH")] JoinedChannel {channel: String, character: CharacterIdentity, title: String},
    #[serde(rename="KID")] Kinks {#[serde(rename="type")] response_type: KinkResponsePart, message: String, key: Vec<u32>, value: Vec<u32>},
    #[serde(rename="LCH")] LeftChannel {channel: String, character: String},
    #[serde(rename="LIS")] ListOnline {characters: Vec<CharacterData>},
    #[serde(rename="NLN")] NewConnection {status: Status, gender: Gender, identity: String},
    #[serde(rename="IGN")] Ignore {action: IgnoreAction, #[serde(default)] characters: Vec<String>, #[serde(default)] character: String}, // Thanks, F-List. 'Characters' only when init/list
    #[serde(rename="FRL")] Friends {characters: String},
    #[serde(rename="ORS")] Channels {channels: Vec<ChannelInfo>},
    #[serde(rename="PIN")] Ping,
    #[serde(rename="PRD")] ProfileData {#[serde(rename="type")] response_type: ProfileDataPart, #[serde(default)] message: String, #[serde(default)] key: String, #[serde(default)] value: String},
    #[serde(rename="PRI")] PrivateMessage {character: String, message: String},
    #[serde(rename="MSG")] Message {character: String, message: String, channel: String},
    #[serde(rename="LRP")] Ad {character: String, message: String, channel: String},
    #[serde(rename="RLL")] Roll {channel: String, results: u32, #[serde(rename="type")] response_type: String, rolls: Vec<String>, character: String, endresult: u32, message: String},
    #[serde(rename="RMO")] ChannelMode {mode: ChannelMode, channel: String},
    #[serde(rename="RTB")] BridgeEvent {#[serde(rename="type")] response_type: String, character: String},
    #[serde(rename="SFC")] Report {action: String, moderator: String, character: String, timestamp: String, callid: u32, report: String, logid: u32}, // May need to flatten an enum in here instead. 
    #[serde(rename="STA")] Status {status: Status, character: String, statusmsg: String},
    #[serde(rename="SYS")] SystemMessage {message: String, channel: String}, // Catch-all response for many things. Fuck this universe.
    #[serde(rename="TPN")] Typing {character: String, status: TypingStatus},
    #[serde(rename="UPT")] Uptime {time: u64, starttime: u64, startstring: String, accepted: u64, channels: u32, users: u32, maxusers: u32},
    #[serde(rename="VAR")] Variable {variable: String, value: serde_json::Value}, // Could be int, float, [string]; I hate it. Use an adjacently tagged enum.
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