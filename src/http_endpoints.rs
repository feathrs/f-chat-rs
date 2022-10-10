use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use crate::{util::{StringBool, StringInteger}, data::{Character, CharacterId, KinkInterest}};
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
pub struct IdItem {
    name: String,
    id: StringInteger
}

#[derive(Deserialize)]
pub struct Kink {
    #[serde(flatten)]
    id: IdItem,
    description: String,
    group_id: StringInteger
}

#[derive(Deserialize)]
pub struct KinkGroup(IdItem);

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
pub struct InfoTagGroup(IdItem);

#[derive(Deserialize)]
pub struct ListItem {
    id: IdItem,
    value: String
}

pub async fn get_mapping_list(client: &Client) -> reqwest::Result<MappingListResponse> {
    let empty_data: HashMap<String, String> = HashMap::new(); // Forgive me, for I am sin.
    client.post("https://www.f-list.net/json/api/mapping-list.php")
        .form(&empty_data)
        .send()
        .await?
        .json()
        .await
}

#[derive(Deserialize)]
pub struct HasError<T> {
    error: String,
    #[serde(flatten)]
    inner: T
}

#[derive(Serialize)]
struct Authenticated<T> {
    account: String,
    ticket: String,
    #[serde(flatten)]
    inner: T
}

#[derive(Serialize)]
#[serde(untagged)]
enum CharacterProfileRequest {
    Name {name: Character},
    Id {id: CharacterId}
}

#[derive(Deserialize)]
pub struct CharacterProfileResponse {
    badges: Vec<String>,
    character_list: Vec<FullCharacter>, // I hate you.
    created_at: u64,
    custom_kinks: HashMap<StringInteger, CustomKink>,
    custom_title: Option<String>,
    customs_first: bool,
    description: String,
    id: u64,
    images: Vec<Image>,
    infotags: Vec<String>, // Dead. Dead to me. Killed.
    inlines: HashMap<StringInteger, Inline>,
    is_self: bool,
    kinks: HashMap<StringInteger, KinkInterest>,
    memo: Memo,
    name: String,
    settings: Settings,
    updated_at: u64,
    views: u64
}

#[derive(Deserialize)]
pub struct FullCharacter(IdItem);

#[derive(Deserialize)]
pub struct CustomKink {
    name: String,
    description: String,
    choice: KinkInterest,
    children: Vec<u64>
}

#[derive(Deserialize)]
pub struct Image {
    description: String,
    extension: String,
    height: String,
    image_id: String,
    sort_order: StringInteger, // Thanks. Clowns.
    width: String
}

#[derive(Deserialize)]
pub struct Inline {
    extension: String,
    hash: String,
    nsfw: bool
}

#[derive(Deserialize)]
pub struct Memo {
    id: u64,
    memo: String
}

#[derive(Deserialize)]
pub struct Settings {
    customs_first: bool,
    show_friends: bool,
    guestbook: bool,
    prevent_bookmarks: bool,
    public: bool
}

pub async fn get_character_id_profile_data(client: &Client, ticket: String, account: String, character_id: CharacterId) -> reqwest::Result<HasError<CharacterProfileResponse>> {
    let data = Authenticated {
        account, ticket,
        inner: CharacterProfileRequest::Id { id: character_id }
    };
    get_character_data_base(client, data).await
}
pub async fn get_character_profile_data(client: &Client, ticket: String, account: String, character: Character) -> reqwest::Result<HasError<CharacterProfileResponse>> {
    let data = Authenticated {
        account, ticket,
        inner: CharacterProfileRequest::Name { name: character }
    };
    get_character_data_base(client, data).await
}
async fn get_character_data_base(client: &Client, req: Authenticated<CharacterProfileRequest>) -> reqwest::Result<HasError<CharacterProfileResponse>> {
    client.post("https://www.f-list.net/json/api/character-data.php")
        .form(&req)
        .send()
        .await?
        .json()
        .await
}