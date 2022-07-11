use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Gender {
    Male,
    Female,
    Transgender,
    Herm,
    Shemale,
    #[serde(rename = "Male-Herm")]
    MaleHerm,
    #[serde(rename = "Cunt-Boy")]
    CBoy, // Look, I don't make the rules.
    None,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelMode {
    #[serde(rename = "chat")]
    ChatOnly,
    #[serde(rename = "ads")]
    AdsOnly,
    Both,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelStatus {
    Public,
    Private,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TypingStatus {
    Clear,
    Paused,
    Typing,
}

// Strong typing for string IDs
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Channel(String);
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Character(String);
