#[test]
fn it_works() {
    let result = 2 + 2;
    assert_eq!(result, 4);
}

#[test]
fn server_command_deserialize() {
    use crate::protocol::{parse_command, ServerCommand, CharacterData};
    use crate::data::{Character, Gender, Status};
    // Taken straight from the Server Commands raw samples
    assert_eq!(
        parse_command(r#"LIS {"characters": [["Alexandrea", "Female", "online", ""], ["Fa Mulan", "Female", "busy", "Away, check out my new alt Aya Kinjou!"], ["Adorkable Lexi", "Female", "online", ""], ["Melfice Cyrum", "Male", "online", ""], ["Jenasys Stryphe", "Female", "online", ""], ["Cassie Hazel", "Herm", "looking", ""], ["Jun Watarase", "Male", "looking", "cute femmy boi looking for a dominate partner"],["Motley Ferret", "Male", "online", ""], ["Tashi", "Male", "online", ""], ["Viol", "Cunt-boy", "looking", ""], ["Dorjan Kazyanenko", "Male", "looking", ""], ["Asaki", "Female", "online", ""]]}"#),
        ServerCommand::ListOnline {
            characters: vec![
                CharacterData(Character("Alexandrea".to_owned()), Gender::Female, Status::Online, "".to_owned())
            ]
        }
    )
}
