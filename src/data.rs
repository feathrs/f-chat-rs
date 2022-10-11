use serde::{Deserialize, Serialize};
use crate::stringable;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ChannelMode {
    #[serde(rename = "chat")]
    ChatOnly,
    #[serde(rename = "ads")]
    AdsOnly,
    Both,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ChannelStatus {
    Public,
    Private,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Online,
    Looking,
    Busy,
    Dnd,
    Idle,
    Away,
    Crown, // If you try to set crown, you will die.
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TypingStatus {
    Clear,
    Paused,
    Typing,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum KinkInterest {
    Fave,
    Yes,
    Maybe,
    No
}

// Strong typing for string IDs
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Channel(pub String);
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Character(pub String);

stringable!(CharacterId: u64, CharacterIdProxy, "CharacterIdProxy");