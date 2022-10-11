use std::collections::HashMap;

use serde::{Serialize, Deserialize, de::DeserializeOwned};
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
pub enum CharacterRequest {
    Name {name: Character},
    Id {id: CharacterId}
}

impl From<Character> for CharacterRequest {
    fn from(name: Character) -> Self {
        CharacterRequest::Name { name }
    }
}

impl From<CharacterId> for CharacterRequest {
    fn from(id: CharacterId) -> Self {
        CharacterRequest::Id { id }
    }
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
    #[serde(alias="id")] // Some later versions of this structure use id instead
    image_id: String,
    sort_order: StringInteger, // Thanks. Clowns.
    width: String,
    url: Option<String> // Included in full response but not profile? Supposedly can be constructed manually.
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

type HasResult<T> = reqwest::Result<HasError<T>>;
pub async fn get_character_base<T: Into<CharacterRequest>, R: DeserializeOwned>(url: &str, client: &Client, ticket: String, account: String, character: T) -> HasResult<R> {
    let data = Authenticated {
        account, ticket,
        inner: character.into()
    };
    client.post(url)
        .form(&data)
        .send()
        .await?
        .json()
        .await
}

macro_rules! character_fn {
    ($url:literal, $i:ident : $t:ty) => {
        pub async fn $i<T: Into<CharacterRequest>>(client: &Client, ticket: String, account: String, character: T) -> HasResult<$t> {
            get_character_base(concat!("https://www.f-list.net", $url), client, ticket, account, character).await
        }
    };
}

character_fn!("/json/api/character-data.php", get_character_profile_data : CharacterProfileResponse);

#[derive(Deserialize)]
pub struct CharacterFriendsResponse {
    friends: Vec<FullCharacter>
}

character_fn!("/json/api/character-friends.php", get_character_friends : CharacterFriendsResponse);

#[derive(Deserialize)]
pub struct CharacterImagesResponse {
    images: Vec<Image>
}

character_fn!("/json/api/character-images.php", get_character_images : CharacterImagesResponse);