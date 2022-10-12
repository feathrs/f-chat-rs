// Design considerations:
// - Use a single shared and managed API ticket. They last for 30 minutes.
// - Support aggregating many WS sessions into a single client
// - Aggregate many character non-specific details
// - Assume that each WS session has consistent state sync.
// - Combine WS sessions and HTTP endpoints into one thing
// - Use a trait implementation for events. Have a generic client.

use std::{time::{Instant, Duration}, sync::Arc};
use bimap::BiBTreeMap;
use thiserror::Error;

use reqwest::Client as ReqwestClient;
use tokio::{net::TcpStream, sync::mpsc::{Sender, Receiver, channel}};
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream, tungstenite::Message};
use futures_util::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};

use crate::{http_endpoints::{get_api_ticket, Bookmark, Friend}, data::{CharacterId, Character}, protocol::*};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug)]
pub struct Client {
    client_name: String,
    client_version: String,

    username: String,
    password: String,
    ticket: String,
    last_ticket: Instant,
    http_client: ReqwestClient,
    default_character: CharacterId,

    characters: BiBTreeMap<Character, CharacterId>,
    bookmarks: Vec<Bookmark>,
    friends: Vec<Friend>,

    sessions: Vec<Arc<Session>>,
    dispatch_channel: (Sender<ServerCommand>, Receiver<ServerCommand>)
}

#[derive(Debug)]
pub struct Session {
    character: Character,
    write: SplitSink<Socket, Message>
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Error from HTTP Request")]
    RequestError(#[from] reqwest::Error),
    #[error("Error from Websocket (Tungstenite)")]
    WebsocketError(#[from] tokio_tungstenite::tungstenite::Error),
}

impl Client {
    pub fn new(username: String, password: String, client_name: String, client_version: String) -> Client {
        let http = ReqwestClient::new();
        Client {
            // Use a builder for this later. Part of a refactoring task.
            client_name, client_version,

            username, password, 
            ticket: "NONE".to_owned(),
            last_ticket: Instant::now() - Duration::from_secs(60*60), // Pretend that the last ticket was an hour ago.
            http_client: http,
            default_character: CharacterId(0),

            characters: BiBTreeMap::new(),
            bookmarks: Vec::new(),
            friends: Vec::new(),

            sessions: Vec::new(),
            dispatch_channel: channel(8),
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

    pub async fn refresh_fast(&mut self) -> Result<(), ClientError> {
        // Optimistically refresh if the token is more than 20 minutes old
        // Supposedly it lasts 30 minutes but I don't trust these devs and their crap API
        if self.last_ticket + Duration::from_secs(20*60) < Instant::now() { return Ok(()) }
        self.refresh().await
    }

    pub async fn connect(&mut self, character: Character) -> Result<(), ClientError> {
        self.refresh_fast().await?;
        let (mut socket, _) = connect_async("wss://chat.f-list.net/chat2").await?;
        
        socket.send(Message::Text(prepare_command(&ClientCommand::Identify { 
            method: IdentifyMethod::Ticket, 
            account: self.username.clone(), 
            ticket: self.ticket.clone(), 
            character: character.clone(), 
            client_name: self.client_name.clone(), 
            client_version: self.client_version.clone() 
        }))).await?;

        let (write, read) = socket.split();

        let session = Arc::new(Session {
            character,
            write,
        });
        self.sessions.push(session.clone());

        // Oh, and something will need to listen to these...
        let chan = self.dispatch_channel.0.clone();
        tokio::spawn(async move {
            read.for_each(move |res| {
                // We don't want this to happen concurrently, because the events need to arrive in order
                // But they only need to arrive in order for any given connection.
                // Connections will end up interleaved in the channel consumer.
                let chan = chan.clone();
                async move {
                    match res {
                        Err(err) => {eprintln!("Error when reading from session: {err:?}");},
                        Ok(ok) => {
                            match ok.to_text() {
                                Err(err) => {eprintln!("Message frame is not text: {err:?}");},
                                Ok(text) => {
                                    let command = parse_command(text);
                                    chan.send(command).await.expect("Failed to send command through Tokio mpsc channel");
                                }
                            };
                        }
                    };
                }
            })
        });

        Ok(())
    }


}