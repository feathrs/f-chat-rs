use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use crate::{util::{StringBool, StringInteger}, data::{Character, CharacterId}};
use reqwest::Client;

#[derive(Serialize)]
struct ApiTicketRequest<'a,'b> {
    account: &'a str,
    password: &'b str,
    no_characters: StringBool,
    no_friends: StringBool,
    no_bookmarks: StringBool,
    new_character_list: StringBool
}

#[derive(Deserialize)]
pub struct ApiTicketResponse {
    bookmarks: Vec<Bookmark>,
    characters: HashMap<Character, CharacterId>,
    default_character: CharacterId,
    friends: Vec<FriendRelation>,
    ticket: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Bookmark {
    name: Character
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct FriendRelation {
    dest_name: Character,
    source_name: Character
}

// Only new-format char list. Includes friends & bookmarks response if extra.
pub async fn get_api_ticket(client: &Client, username: &str, password: &str, extra: bool) -> reqwest::Result<ApiTicketResponse> {
    let body = ApiTicketRequest {
        account: username,
        password: password,
        no_characters: StringBool(!extra),
        no_friends: StringBool(!extra),
        no_bookmarks: StringBool(!extra),
        new_character_list: StringBool(extra),
    };
    client.post("https://www.f-list.net/json/getApiTicket.php")
        .form(&body)
        .send()
        .await?
        .json()
        .await
} 

#[derive(Deserialize)]
pub struct MappingListResponse {
    kinks: Vec<Kink>,
    kink_groups: Vec<KinkGroup>,
    infotags: Vec<InfoTag>,
    infotags_groups: Vec<InfoTagGroup>,
    listitems: Vec<ListItem>
}

#[derive(Deserialize)]
pub struct Kink {
    description: String,
    group_id: StringInteger,
    id: StringInteger,
    name: String
}

#[derive(Deserialize)]
pub struct KinkGroup {
    id: StringInteger,
    name: String
}

#[derive(Deserialize)]
pub struct InfoTag {
    group_id: StringInteger,
    id: StringInteger,
    list: String,
    name: String,
    #[serde(rename="type")]
    tag_type: InfoTagType
}

#[derive(Deserialize)]
pub enum InfoTagType {
    #[serde(rename="text")]
    Text,
    #[serde(rename="number")]
    Number,
    #[serde(rename="list")]
    List
}

#[derive(Deserialize)]
pub struct InfoTagGroup {
    id: StringInteger,
    name: String
}

#[derive(Deserialize)]
pub struct ListItem {
    id: StringInteger,
    name: String,
    value: String
}

pub async fn get_mapping_list(client: &Client, auth: &str) -> reqwest::Result<MappingListResponse> {
    let empty_data: HashMap<String, String> = HashMap::new(); // Forgive me, for I am sin.
    client.post("https://www.f-list.net/json/api/mapping-list.php")
        .form(&empty_data)
        .send()
        .await?
        .json()
        .await
}

