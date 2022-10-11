// Design considerations:
// - Use a single shared and managed API ticket. They last for 30 minutes.
// - Support aggregating many WS sessions into a single client
// - Aggregate many character non-specific details
// - Assume that each WS session has consistent state sync.
// - Combine WS sessions and HTTP endpoints into one thing
// - Use a trait implementation for events. Have a generic client.

use std::time::{Instant, Duration};
use bimap::BiBTreeMap;
use thiserror::Error;

use reqwest::Client as ReqwestClient;

use crate::{http_endpoints::{get_api_ticket, Bookmark, Friend}, data::{CharacterId, Character}};

#[derive(Debug)]
pub struct Client {
    username: String,
    password: String,
    ticket: String,
    last_ticket: Instant,
    http_client: ReqwestClient,
    default_character: CharacterId,

    characters: BiBTreeMap<Character, CharacterId>,
    bookmarks: Vec<Bookmark>,
    friends: Vec<Friend>
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Error from HTTP Request")]
    RequestError(#[from] reqwest::Error)
}

impl Client {
    pub fn new(username: String, password: String) -> Client {
        let http = ReqwestClient::new();
        Client {
            username, password, 
            ticket: "NONE".to_owned(),
            last_ticket: Instant::now() - Duration::from_secs(60*60), // Pretend that the last ticket was an hour ago.
            http_client: http,
            default_character: CharacterId(0),

            characters: BiBTreeMap::new(),
            bookmarks: Vec::new(),
            friends: Vec::new()
        }
    }

    pub async fn init(&mut self) -> Result<(), ClientError> {
        let mut ticket = get_api_ticket(&self.http_client, &self.username, &self.password, true).await?;
        self.ticket = ticket.ticket;
        self.last_ticket = Instant::now();
        self.default_character = ticket.default_character;

        self.characters = ticket.characters.drain().collect();
        self.bookmarks = ticket.bookmarks;
        self.friends = ticket.friends;

        Ok(())
    }

    pub async fn refresh(&mut self) -> Result<(), ClientError> {
        let ticket = get_api_ticket(&self.http_client, &self.username, &self.password, false).await?;
        self.ticket = ticket.ticket;
        self.last_ticket = Instant::now();
        Ok(())
    }
}