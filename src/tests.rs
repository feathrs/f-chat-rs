#[test]
fn server_command_deserialize() {
    use crate::protocol::{parse_command, ServerCommand, FlatCharacterData};
    use crate::data::{Character, Gender, Status};
    // Taken straight from the Server Commands raw samples
    assert_eq!(
        parse_command(r#"LIS {"characters": [["Alexandrea", "Female", "online", ""], ["Fa Mulan", "Female", "busy", "Away, check out my new alt Aya Kinjou!"], ["Adorkable Lexi", "Female", "online", ""], ["Melfice Cyrum", "Male", "online", ""], ["Jenasys Stryphe", "Female", "online", ""], ["Cassie Hazel", "Herm", "looking", ""], ["Viol", "Cunt-boy", "looking", ""]]}"#),
        ServerCommand::ListOnline {
            characters: vec![
                FlatCharacterData(Character("Alexandrea".to_owned()), Gender::Female, Status::Online, "".to_owned()),
                FlatCharacterData(Character("Fa Mulan".to_owned()), Gender::Female, Status::Busy, "Away, check out my new alt Aya Kinjou!".to_owned()),
                FlatCharacterData(Character("Adorkable Lexi".to_owned()), Gender::Female, Status::Online, "".to_owned()),
                FlatCharacterData(Character("Melfice Cyrum".to_owned()), Gender::Male, Status::Online, "".to_owned()),
                FlatCharacterData(Character("Jenasys Stryphe".to_owned()), Gender::Female, Status::Online, "".to_owned()),
                FlatCharacterData(Character("Cassie Hazel".to_owned()), Gender::Herm, Status::Looking, "".to_owned()),
                FlatCharacterData(Character("Viol".to_owned()), Gender::CBoy, Status::Looking, "".to_owned()),
            ]
        }
    )
}

#[test]
fn client_command_serialize() {
    use crate::protocol::{prepare_command, ClientCommand, KinkId};
    use crate::data::{Gender, Orientation, Language, FurryPreference, Role};
    assert_eq!(
        r#"FKS {"furryprefs":["Furs and / or humans","Humans ok, Furries Preferred","No humans, just furry characters"],"genders":["Male","Male-Herm"],"kinks":["523","66"],"languages":["Dutch"],"orientations":["Gay","Bi - male preference","Bisexual"],"roles":["Always dominant","Usually dominant"]}"#,
        prepare_command(&ClientCommand::Search {
            kinks: vec![KinkId(523),KinkId(66)],
            genders: vec![Gender::Male, Gender::MaleHerm], 
            orientations: vec![Orientation::Gay, Orientation::BiMalePref, Orientation::Bisexual], 
            languages: vec![Language::Dutch], 
            furryprefs: vec![FurryPreference::Both, FurryPreference::FurryPref, FurryPreference::FurryOnly], 
            roles: vec![Role::AlwaysDom, Role::UsuallyDom] 
        })
    )
}

#[test]
fn test_kink_serde() {
    use crate::protocol::{KinkId};
    use serde_json::{to_string, from_str};
    assert_eq!(r#""621""#, to_string(&KinkId(621)).expect("Failed to serialize"));
    assert_eq!(KinkId(621), from_str::<KinkId>(r#""621""#).expect("Failed to deserialize from string"));
    assert_eq!(KinkId(621), from_str::<KinkId>("621").expect("Failed to deserialize from number"));
}

#[test]
fn test_string_bool() {
    use crate::util::StringBool;
    use serde_json::{to_string, from_str};
    assert_eq!(r#""true""#, to_string(&StringBool(true)).expect("Failed to serialize 'true'"));
    assert_eq!(r#""false""#, to_string(&StringBool(false)).expect("Failed to serialize 'false'"));
    assert_eq!(StringBool(true), from_str::<StringBool>("true").expect("Failed to deserialize from bool"));
    assert_eq!(StringBool(true), from_str::<StringBool>(r#""true""#).expect("Failed to deserialize from string"));
}

#[test]
fn test_string_integer() {
    use crate::util::StringInteger;
    use serde_json::{to_string, from_str};
    assert_eq!(r#""621""#, to_string(&StringInteger(621)).expect("Failed to serialize '621'"));
    assert_eq!(StringInteger(621), from_str::<StringInteger>("621").expect("Failed to deserialize from number"));
    assert_eq!(StringInteger(621), from_str::<StringInteger>(r#""621""#).expect("Failed to deserialize from string"));
}