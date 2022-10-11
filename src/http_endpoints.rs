use std::collections::HashMap;

use serde::{Serialize, Deserialize, de::DeserializeOwned};
use crate::{util::{StringBool, StringInteger}, data::{Character, CharacterId, KinkInterest, Channel}};
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

#[derive(Deserialize, Debug)]
pub struct ApiTicketResponse {
    pub bookmarks: Vec<Bookmark>,
    pub characters: HashMap<Character, CharacterId>,
    pub default_character: CharacterId,
    pub friends: Vec<Friend>,
    pub ticket: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Bookmark {
    pub name: Character
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

#[derive(Deserialize, Debug)]
pub struct MappingListResponse {
    pub kinks: Vec<Kink>,
    pub kink_groups: Vec<KinkGroup>,
    pub infotags: Vec<InfoTag>,
    pub infotags_groups: Vec<InfoTagGroup>,
    pub listitems: Vec<ListItem>
}

#[derive(Deserialize, Debug)]
pub struct IdItem {
    pub name: String,
    pub id: StringInteger
}

#[derive(Deserialize, Debug)]
pub struct Kink {
    #[serde(flatten)]
    pub id: IdItem,
    pub description: String,
    pub group_id: StringInteger
}

#[derive(Deserialize, Debug)]
pub struct KinkGroup(pub IdItem);

#[derive(Deserialize, Debug)]
pub struct InfoTag {
    pub group_id: StringInteger,
    pub id: StringInteger,
    pub list: String,
    pub name: String,
    #[serde(rename="type")]
    pub tag_type: InfoTagType
}

#[derive(Deserialize, Debug)]
pub enum InfoTagType {
    #[serde(rename="text")]
    Text,
    #[serde(rename="number")]
    Number,
    #[serde(rename="list")]
    List
}

#[derive(Deserialize, Debug)]
pub struct InfoTagGroup(pub IdItem);

#[derive(Deserialize, Debug)]
pub struct ListItem {
    pub id: IdItem,
    pub value: String
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

#[derive(Deserialize, Debug)]
pub struct HasError<T> {
    #[serde(default)]
    pub error: String,
    #[serde(flatten)]
    pub inner: T
}

#[derive(Serialize)]
struct Authenticated<'a,'b,T> {
    pub account: &'a str,
    pub ticket: &'b str,
    #[serde(flatten)]
    pub inner: T
}

#[derive(Serialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct CharacterProfileResponse {
    pub badges: Vec<String>,
    pub character_list: Vec<FullCharacter>, // I hate you.
    pub created_at: u64,
    pub custom_kinks: HashMap<StringInteger, CustomKink>,
    pub custom_title: Option<String>,
    pub customs_first: bool,
    pub description: String,
    pub id: u64,
    pub images: Vec<Image>,
    pub infotags: Vec<String>, // Dead. Dead to me. Killed.
    pub inlines: HashMap<StringInteger, Inline>,
    pub is_self: bool,
    pub kinks: HashMap<StringInteger, KinkInterest>,
    pub memo: Memo,
    pub name: String,
    pub settings: Settings,
    pub updated_at: u64,
    pub views: u64
}

#[derive(Deserialize, Debug)]
pub struct FullCharacter(pub IdItem);

#[derive(Deserialize, Debug)]
pub struct CustomKink {
    pub name: String,
    pub description: String,
    pub choice: KinkInterest,
    pub children: Vec<u64>
}

#[derive(Deserialize, Debug)]
pub struct Image {
    pub description: String,
    pub extension: String,
    pub height: String,
    #[serde(alias="id")] // Some later versions of this structure use id instead
    pub image_id: String,
    pub sort_order: StringInteger, // Thanks. Clowns.
    pub width: String,
    pub url: Option<String> // Included in full response but not profile? Supposedly can be constructed manually.
}

#[derive(Deserialize, Debug)]
pub struct Inline {
    pub extension: String,
    pub hash: String,
    pub nsfw: bool
}

#[derive(Deserialize, Debug)]
pub struct Memo {
    pub id: u64,
    pub memo: String
}

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub customs_first: bool,
    pub show_friends: bool,
    pub guestbook: bool,
    pub prevent_bookmarks: bool,
    pub public: bool
}

type HasResult<T> = reqwest::Result<HasError<T>>;

pub async fn req_base<T: Serialize, R: DeserializeOwned>(url: &str, client: &Client, data: T) -> HasResult<R> {
    client.post(url)
        .form(&data)
        .send()
        .await?
        .json()
        .await
}

pub async fn get_character_base<T: Into<CharacterRequest>, R: DeserializeOwned>(url: &str, client: &Client, ticket: &str, account: &str, character: T) -> HasResult<R> {
    let data = Authenticated {
        account, ticket,
        inner: character.into()
    };
    req_base(url, client, data).await
}

macro_rules! character_fn {
    ($url:literal, $i:ident : $t:ty) => {
        pub async fn $i<T: Into<CharacterRequest>>(client: &Client, ticket: &str, account: &str, character: T) -> HasResult<$t> {
            get_character_base(concat!("https://www.f-list.net", $url), client, ticket, account, character).await
        }
    };
}

character_fn!("/json/api/character-data.php", get_character_profile_data : CharacterProfileResponse);

#[derive(Deserialize, Debug)]
pub struct CharacterFriendsResponse {
    pub friends: Vec<FullCharacter>
}

character_fn!("/json/api/character-friends.php", get_character_friends : CharacterFriendsResponse);

#[derive(Deserialize, Debug)]
pub struct CharacterImagesResponse {
    pub images: Vec<Image>
}

character_fn!("/json/api/character-images.php", get_character_images : CharacterImagesResponse);

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum MemoCharacterRequest { // I hate these people.
    Target { target: Character },
    TargetId { target_id: CharacterId }
}

impl From<CharacterRequest> for MemoCharacterRequest {
    fn from(req: CharacterRequest) -> Self {
        match req {
            CharacterRequest::Id { id } => Self::TargetId { target_id: id },
            CharacterRequest::Name { name } => Self::Target { target: name }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct CharacterMemoResponse {
    pub id: u64,
    pub note: String
}

pub async fn get_character_memo<T: Into<CharacterRequest>>(client: &Client, ticket: &str, account: &str, character: T) -> HasResult<CharacterMemoResponse> {
    let data: Authenticated<MemoCharacterRequest> = Authenticated {
        account, ticket,
        inner: character.into().into()
    };
    req_base(concat!("https://www.f-list.net", "/json/api/character-memo-get2.php"), client, data).await
}

#[derive(Serialize)]
struct SaveCharacterMemoRequest<'a> {
    #[serde(flatten)]
    character: MemoCharacterRequest,
    note: &'a str
}

#[derive(Deserialize, Debug)]
pub struct SaveCharacterMemoResponse {
    pub note: String
}

pub async fn set_character_memo<T: Into<CharacterRequest>>(client: &Client, ticket: &str, account: &str, character: T, memo: &str) -> HasResult<SaveCharacterMemoResponse> {
    let data: Authenticated<SaveCharacterMemoRequest> = Authenticated {
        account, ticket, 
        inner: SaveCharacterMemoRequest { 
            character: character.into().into(),
            note: memo
        }
    };
    req_base(concat!("https://www.f-list.net", "/json/api/character-memo-save.php"), client, data).await
}

#[derive(Serialize)]
struct CharacterGuestbookRequest {
    #[serde(flatten)]
    character: CharacterRequest,
    page: u64
}

#[derive(Deserialize, Debug)]
pub struct CharacterGuestbookResponse {
    pub page: u64, 
    #[serde(rename="canEdit")]
    pub can_edit: bool,
    #[serde(rename="nextPage")]
    pub next_page: bool,
    pub posts: Vec<GuestbookPost>
}

#[derive(Deserialize, Debug)]
pub struct GuestbookPost {
    pub approved: bool,
    #[serde(rename="canEdit")]
    pub can_edit: bool,
    pub character: FullCharacter,
    pub id: u64,
    pub message: String,
    #[serde(rename="postedAt")]
    pub posted_at: u64,
    pub private: bool,
    pub reply: Option<String>
}

pub async fn get_character_guestbook<T: Into<CharacterRequest>>(client: &Client, ticket: &str, account: &str, character: T, page: u64) -> HasResult<CharacterGuestbookResponse> {
    let data = Authenticated {
        account, ticket,
        inner: CharacterGuestbookRequest {
            page, character: character.into().into()
        }
    };
    req_base(concat!("https://www.f-list.net", "/json/api/character-guestbook.php"), client, data).await
}

#[derive(Serialize)]
struct FriendListRequest {
    #[serde(rename = "bookmarklist")]
    bookmarks: StringBool,
    #[serde(rename = "friendlist")]
    friends: StringBool,
    #[serde(rename = "requestlist")]
    pending_incoming: StringBool,
    #[serde(rename = "requestpending")]
    pending_outgoing: StringBool,
}

#[derive(Deserialize, Debug)]
pub struct FriendListResponse {
    #[serde(rename = "bookmarklist")]
    pub bookmarks: Vec<Character>,
    #[serde(rename = "friendlist")]
    pub friends: Vec<Friend>,
    #[serde(rename = "requestlist")]
    pub pending_incoming: Vec<FriendRequest>,
    #[serde(rename = "requestpending")]
    pub pending_outgoing: Vec<FriendRequest>,
}

#[derive(Deserialize, Debug)]
pub struct Friend {
    #[serde(alias="dest_name")]
    pub dest: Character,
    #[serde(default)]
    pub last_online: u64, // Seconds since last online -- Not timestamp of last online?
    #[serde(alias="source_name")]
    pub source: Character
}

#[derive(Deserialize, Debug)]
pub struct FriendRequest {
    pub dest: Character,
    pub id: u64,
    pub source: Character
}

pub async fn get_friends_list(client: &Client, ticket: &str, account: &str) -> HasResult<FriendListResponse> {
    let data = Authenticated {
        account, ticket,
        inner: FriendListRequest {
            bookmarks: StringBool(true),
            friends: StringBool(true),
            pending_incoming: StringBool(true),
            pending_outgoing: StringBool(true)
        }
    };
    req_base(concat!("https://www.f-list.net", "/json/api/friend-bookmark-lists.php"), client, data).await
}

#[derive(Deserialize, Debug)]
pub struct EmptyResponse {}

character_fn!("/json/api/bookmark-add.php", add_bookmark: EmptyResponse);
character_fn!("/json/api/bookmark-remove.php", remove_bookmark: EmptyResponse);

#[derive(Serialize)]
struct RemoveFriendRequest { // Nice.
    #[serde(flatten)]
    source: SourceRequest,
    #[serde(flatten)]
    dest: DestRequest
}

#[derive(Serialize)]
#[serde(untagged)]
enum SourceRequest {
    Character { source_name: Character },
    CharacterId { source_id: CharacterId }
}

#[derive(Serialize)]
#[serde(untagged)]
enum DestRequest {
    Character { dest_name: Character },
    CharacterId { dest_id: CharacterId }
}

impl From<CharacterRequest> for SourceRequest {
    fn from(req: CharacterRequest) -> Self {
        match req {
            CharacterRequest::Name { name } => SourceRequest::Character { source_name: name },
            CharacterRequest::Id { id } => SourceRequest::CharacterId { source_id: id },
        }
    }
}

impl From<CharacterRequest> for DestRequest {
    fn from(req: CharacterRequest) -> Self {
        match req {
            CharacterRequest::Name { name } => DestRequest::Character { dest_name: name },
            CharacterRequest::Id { id } => DestRequest::CharacterId { dest_id: id },
        }
    }
}

pub async fn remove_friend<T1: Into<CharacterRequest>, T2: Into<CharacterRequest>>(client: &Client, ticket: &str, account: &str, source: T1, dest: T2) -> HasResult<EmptyResponse> {
    let data = Authenticated {
        account, ticket,
        inner: RemoveFriendRequest {
            source: source.into().into(),
            dest: dest.into().into()
        }
    };
    req_base(concat!("https://www.f-list.net", "/json/api/friend-remove.php"), client, data).await
}

#[derive(Serialize)]
struct FriendRequestId {
    request_id: u64
}

pub async fn accept_friend_request(client: &Client, ticket: &str, account: &str, request: u64) -> HasResult<EmptyResponse> {
    let data = Authenticated {
        account, ticket,
        inner: FriendRequestId {
            request_id: request
        }
    };
    req_base(concat!("https://www.f-list.net", "/json/api/request-accept.php"), client, data).await
}

pub async fn deny_friend_request(client: &Client, ticket: &str, account: &str, request: u64) -> HasResult<EmptyResponse> {
    let data = Authenticated {
        account, ticket,
        inner: FriendRequestId {
            request_id: request
        }
    };
    req_base(concat!("https://www.f-list.net", "/json/api/request-deny.php"), client, data).await
}

pub async fn cancel_friend_request(client: &Client, ticket: &str, account: &str, request: u64) -> HasResult<EmptyResponse> {
    let data = Authenticated {
        account, ticket,
        inner: FriendRequestId {
            request_id: request
        }
    };
    req_base(concat!("https://www.f-list.net", "/json/api/request-cancel.php"), client, data).await
}

#[derive(Serialize)]
struct FriendRequestRequest {
    source: SourceRequest,
    target: TargetRequest
}

#[derive(Serialize)]
#[serde(untagged)]
enum TargetRequest { // I'm going to kill people. This is a threat.
    Character { target_name: Character },
    CharacterId { target_id: CharacterId }
}

#[derive(Deserialize, Debug)]
pub struct FriendRequestResponse {
    pub request: FriendRequestPartial
}

#[derive(Deserialize, Debug)]
pub struct FriendRequestPartial {
    pub id: u64,
    pub source: FullCharacter,
    pub target: FullCharacter,
    #[serde(rename="createdAt")]
    pub created_at: u64
}

impl From<CharacterRequest> for TargetRequest {
    fn from(req: CharacterRequest) -> Self {
        match req {
            CharacterRequest::Name { name } => TargetRequest::Character { target_name: name },
            CharacterRequest::Id { id } => TargetRequest::CharacterId { target_id: id },
        }
    }
}

pub async fn send_friend_request<T1: Into<CharacterRequest>, T2: Into<CharacterRequest>>(client: &Client, ticket: &str, account: &str, source: T1, target: T2) -> HasResult<FriendRequestResponse> {
    let data = Authenticated {
        account, ticket,
        inner: FriendRequestRequest {
            source: source.into().into(),
            target: target.into().into()
        }
    };
    req_base(concat!("https://www.f-list.net", "/json/api/request-send2.php"), client, data).await
}

#[derive(Serialize)]
struct ReportRequest<'a,'b> {
    character: Character,
    #[serde(flatten)]
    target: ReportTarget,
    #[serde(rename="reportText")]
    report_text: &'a str,
    log: &'b str,
    text: StringBool // Must be "true". Always. 
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum ReportTarget {
    Character { #[serde(rename="reportUser")] character: Character },
    Channel { channel: Channel }
}

impl From<Channel> for ReportTarget {
    fn from(chan: Channel) -> Self {
        Self::Channel { channel: chan }
    }
}

impl From<Character> for ReportTarget {
    fn from(character: Character) -> Self {
        Self::Character { character }
    }
}

#[derive(Deserialize, Debug)]
pub struct ReportResponse {
    pub log_id: StringInteger
}

pub async fn report<T: Into<ReportTarget>>(client: &Client, ticket: &str, account: &str, from: Character, target: T, reason: &str, log: &str) -> HasResult<ReportResponse> {
    let data = Authenticated {
        account, ticket,
        inner: ReportRequest {
            character: from,
            target: target.into(),
            report_text: reason,
            log,
            text: StringBool(true)
        }
    };
    req_base(concat!("https://www.f-list.net", "/json/api/report-submit.php"), client, data).await
}