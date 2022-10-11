use std::collections::HashMap;

use serde::{Serialize, Deserialize, de::DeserializeOwned};
use crate::{util::{StringBool, StringInteger}, data::{Character, CharacterId, KinkInterest, Channel}, client};
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
    #[serde(default)]
    error: String,
    #[serde(flatten)]
    inner: T
}

#[derive(Serialize)]
struct Authenticated<'a,'b,T> {
    account: &'a str,
    ticket: &'b str,
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

#[derive(Serialize)]
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

#[derive(Deserialize)]
pub struct CharacterMemoResponse {
    id: u64,
    note: String
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

#[derive(Deserialize)]
pub struct SaveCharacterMemoResponse {
    note: String
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

#[derive(Deserialize)]
pub struct CharacterGuestbookResponse {
    page: u64, 
    #[serde(rename="canEdit")]
    can_edit: bool,
    #[serde(rename="nextPage")]
    next_page: bool,
    posts: Vec<GuestbookPost>
}

#[derive(Deserialize)]
pub struct GuestbookPost {
    approved: bool,
    #[serde(rename="canEdit")]
    can_edit: bool,
    character: FullCharacter,
    id: u64,
    message: String,
    #[serde(rename="postedAt")]
    posted_at: u64,
    private: bool,
    reply: Option<String>
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

#[derive(Deserialize)]
pub struct FriendListResponse {
    #[serde(rename = "bookmarklist")]
    bookmarks: Vec<Character>,
    #[serde(rename = "friendlist")]
    friends: Vec<Friend>,
    #[serde(rename = "requestlist")]
    pending_incoming: Vec<FriendRequest>,
    #[serde(rename = "requestpending")]
    pending_outgoing: Vec<FriendRequest>,
}

#[derive(Deserialize)]
pub struct Friend {
    dest: Character,
    last_online: u64, // Seconds since last online -- Not timestamp of last online?
    source: Character
}

#[derive(Deserialize)]
pub struct FriendRequest {
    dest: Character,
    id: u64,
    source: Character
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

#[derive(Deserialize)]
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

#[derive(Deserialize)]
struct FriendRequestResponse {
    request: FriendRequestPartial
}

#[derive(Deserialize)]
struct FriendRequestPartial {
    id: u64,
    source: FullCharacter,
    target: FullCharacter,
    createdAt: u64
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

#[derive(Serialize)]
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

#[derive(Deserialize)]
pub struct ReportResponse {
    log_id: StringInteger
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