use serde::{Deserialize, Serialize};
use crate::stringable;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone)]
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone)]
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone)]
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone)]
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone)]
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone)]
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone)]
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Online,
    Looking,
    Busy,
    Dnd,
    Idle,
    Away,
    Crown, // If you try to set crown, you will die.
    Offline // Also this isn't transmitted by the server but it's sane and internal.
}
impl Default for Status {
    fn default() -> Self {
        Self::Online // Assume online unless *explicitly* Offline.
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone)]
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum KinkInterest {
    Fave,
    Yes,
    Maybe,
    No
}
// No default impl for KinkInterest because it's not sane; unlisted kinks should never be in a collection.

// Strong typing for string IDs
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Channel(pub String);
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Character(pub String);

stringable!(CharacterId: u64, CharacterIdProxy, "CharacterIdProxy");