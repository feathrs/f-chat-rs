#![forbid(private_in_public)]

use crate::{
    stringable,
    util::{timestamp::Timestamp, StackString},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum Gender {
    Male,
    Female,
    Transgender,
    Herm,
    Shemale,
    #[serde(rename = "Male-Herm")]
    MaleHerm,
    #[serde(rename = "Cunt-boy")]
    CBoy, // Look, I don't make the rules.
    None,
}
impl Default for Gender {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum Orientation {
    Straight,
    Gay,
    Bisexual,
    Asexual,
    Unsure,
    #[serde(rename = "Bi - male preference")]
    BiMalePref,
    #[serde(rename = "Bi - female preference")]
    BiFemalePref,
    Pansexual,
    #[serde(rename = "Bi-curious")]
    Bicurious,
}
impl Default for Orientation {
    fn default() -> Self {
        Self::Unsure
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum Language {
    Dutch,
    English,
    French,
    Spanish,
    German,
    Russian,
    Chinese,
    Japanese,
    Portuguese,
    Korean,
    Arabic,
    Italian,
    Swedish,
    Other,
}
impl Default for Language {
    fn default() -> Self {
        Self::English // Other? No. English.
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone, Hash)]
pub enum FurryPreference {
    #[serde(rename = "No furry characters, just humans")]
    HumanOnly,
    #[serde(rename = "Furries ok, Humans Preferred")]
    HumanPref,
    #[serde(rename = "Furs and / or humans")]
    Both,
    #[serde(rename = "Humans ok, Furries Preferred")]
    FurryPref,
    #[serde(rename = "No humans, just furry characters")]
    FurryOnly,
}
impl Default for FurryPreference {
    fn default() -> Self {
        Self::Both
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum Role {
    #[serde(rename = "Always dominant")]
    AlwaysDom,
    #[serde(rename = "Usually dominant")]
    UsuallyDom,
    Switch,
    #[serde(rename = "Usually submissive")]
    UsuallySub,
    #[serde(rename = "Always submissive")]
    AlwaysSub,
    None,
}
impl Default for Role {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ChannelMode {
    #[serde(rename = "chat")]
    ChatOnly,
    #[serde(rename = "ads")]
    AdsOnly,
    Both,
}
impl Default for ChannelMode {
    fn default() -> Self {
        Self::Both
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ChannelStatus {
    Public,
    Private,
}
impl Default for ChannelStatus {
    fn default() -> Self {
        Self::Public
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Online,
    Looking,
    Busy,
    Dnd,
    Idle,
    Away,
    Crown,   // If you try to set crown, you will die.
    Offline, // Also this isn't transmitted by the server but it's sane and internal.
}
impl Default for Status {
    fn default() -> Self {
        Self::Online // Assume online unless *explicitly* Offline.
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone, Hash)]
#[serde(rename_all = "lowercase")]
pub enum TypingStatus {
    Clear,
    Paused,
    Typing,
}
impl Default for TypingStatus {
    fn default() -> Self {
        Self::Clear
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone, Hash)]
pub struct FriendRelation {
    pub own_character: Character,
    pub other_character: Character,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone, Hash)]
#[serde(rename_all = "lowercase")]
pub enum KinkInterest {
    Fave,
    Yes,
    Maybe,
    No,
}
// No default impl for KinkInterest because it's not sane; unlisted kinks should never be in a collection.

// Strong typing for string IDs
// Protocol can't decide if it should use uppercase or lowercase names, so eq case-invariant.

// Channels (IDs) are limited to ~26 characters
// Unofficial channels are 24 chars total
// ADH- 731af796f59d06bec167
// 1234 12345678901234567890

// Longest official channel is "Pregnancy and Impregnation" at 26 chars
//                              12345678901234567890123456
// Don't @ me, I don't make the titles.

// Channel -names- are limited to 64 characters ("64.4999" per error message)
// Accordingly, this may later just become String again.
#[derive(Serialize, Deserialize, Default, Copy, Clone, PartialOrd, Ord, Debug)]
pub struct Channel(pub StackString<26>);
impl PartialEq for Channel {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&*other.0)
    }
}
impl Eq for Channel {}
impl std::hash::Hash for Channel {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_ascii_lowercase().hash(state);
    }
}

// Characters are limited to 20 chars on creation.
// If this gets changed, everything is going to explode. Whoops.
#[derive(Serialize, Deserialize, Default, Copy, Clone, PartialOrd, Ord, Debug)]
pub struct Character(pub StackString<20>);
impl PartialEq for Character {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&*other.0)
    }
}
impl Eq for Character {}
impl std::hash::Hash for Character {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Optimize later with extra StackString? Probably not.
        // Would avoid intermediate heap allocation for String.
        self.0.to_ascii_lowercase().hash(state);
    }
}

stringable!(CharacterId: u64, CharacterIdProxy, "CharacterIdProxy");

// Abstraction for unifying message streams
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MessageChannel {
    Channel(Channel),
    PrivateMessage(Character, Character),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Message {
    #[serde(with = "crate::util::timestamp")]
    pub timestamp: Timestamp,
    pub character: Character,
    pub content: MessageContent,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum MessageContent {
    Message(String),
    Emote(String),
    Roll(Vec<String>, Vec<i32>, i32),
    Bottle(Character),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct ChannelData {
    pub channel: Channel,
    pub channel_mode: ChannelMode,
    pub members: Vec<Character>,
    pub description: String,
    pub title: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct CharacterData {
    pub character: Character,
    pub gender: Gender,
    pub status: Status,
    pub status_message: String,
}
