// Design considerations:
// - Use a single shared and managed API ticket. They last for 30 minutes.
// - Support aggregating many WS sessions into a single client
// - Aggregate many character non-specific details
// - Assume that each WS session has consistent state sync.
// - Combine WS sessions and HTTP endpoints into one thing
// - Use a trait implementation for events. Have a generic client.

// Because this is async, I need two sets of sync/locking objects.
// If I never lock across an await, I can safely* use blocking locks from parking_lot
// Otherwise, I should use the locks from Tokio, or I can cause a deadlock.
// I should also use async locks if contention is high or leases are long.

use std::{time::{Instant, Duration}, sync::Arc, collections::{BTreeSet, BTreeMap}};
use bimap::BiBTreeMap;
use dashmap::{DashMap, DashSet};
use parking_lot::{RwLock, Mutex};
use thiserror::Error;

use reqwest::Client as ReqwestClient;
use tokio::{net::TcpStream, sync::{mpsc::{Sender, Receiver, channel}, Mutex as AsyncMutex}, task::JoinHandle};
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream, tungstenite::{Message, protocol::WebSocketConfig}, connect_async_tls_with_config};
use futures_util::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};

use crate::{http_endpoints::{get_api_ticket, Bookmark, Friend}, data::{CharacterId, Character, Channel, ChannelMode, Gender, Status, TypingStatus}, protocol::*};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug)]
pub struct Client<T: EventListener> {
    client_name: String,
    client_version: String,

    username: String,
    password: String,
    ticket: RwLock<Token>,
    http_client: ReqwestClient,
    default_character: CharacterId,

    // It might later be sane to move this into a specialized cache/state structure
    // That way I can hide the implementation of the caching from consumers.
    // For now, however, I need to understand how I'm using the data.
    // Own account data
    pub own_characters: BiBTreeMap<Character, CharacterId>,
    pub bookmarks: Vec<Bookmark>,
    pub friends: RwLock<Vec<Character>>,

    // Global data
    // I am using DashMap because I don't want to deal with locking, but this might be better as a BTreeMap
    // if I feel like handling locking myself at some point.
    pub channel_data: DashMap<Channel, ChannelData>,
    pub character_data: DashMap<Character, CharacterData>,
    pub admins: RwLock<Vec<Character>>,
    pub ignorelist: RwLock<Vec<Character>>,
    pub global_channels: DashSet<Channel>,

    sessions: RwLock<Vec<Arc<Session>>>,
    send_channel: Sender<Event>,
    rcv_channel: AsyncMutex<Receiver<Event>>,

    event_listener: T
}

#[derive(Debug)]
pub struct Session {
    pub character: Character,
    pub channels: BTreeSet<Channel>,
    pub pms: DashMap<Character, TypingStatus>,
    write: AsyncMutex<SplitSink<Socket, Message>>
}

impl Session {
    pub async fn send(&self, command: ClientCommand) -> Result<(), ClientError> {
        Ok(self.write.lock().await.send(Message::Text(prepare_command(&command))).await?)
    }
}

#[derive(Debug)]
struct Event {
    session: Arc<Session>,
    command: ServerCommand
}

/// Full channel data; everything that describes the channel, inc. members.
#[derive(Debug, Default)]
pub struct ChannelData {
    channel_mode: ChannelMode,
    members: BTreeSet<Character>,
    description: String,
    title: String,

}

#[derive(Debug, Default)]
pub struct CharacterData {
    pub gender: Gender,
    pub status: Status,
    pub status_message: String,
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Error from HTTP Request")]
    RequestError(#[from] reqwest::Error),
    #[error("Error from Websocket (Tungstenite)")]
    WebsocketError(#[from] tokio_tungstenite::tungstenite::Error),
}

#[derive(Debug, Clone)]
struct Token {
    last_updated: Instant,
    pub ticket: String
}

impl Token {
    fn new(ticket: String) -> Token {
        Token {
            ticket, last_updated: Instant::now()
        }
    }
    fn zero() -> Token {
        Token {
            ticket: "NONE".to_owned(),
            last_updated: Instant::now() - Duration::from_secs(60*60) // Pretend that the last ticket was an hour ago.
        }
    }
    fn expired(&self) -> bool {
        self.last_updated.elapsed() > Duration::from_secs(25*60)
    }
    fn update(&mut self, new: String) {
        self.ticket = new;
        self.last_updated = Instant::now();
    }
}

impl<T: EventListener> Client<T> {
    pub fn new(username: String, password: String, client_name: String, client_version: String, event_listener: T) -> Client<T> {
        let http = ReqwestClient::new();
        let (send, rcv) = channel(8);
        Client {
            // Use a builder for this later. Part of a refactoring task.
            client_name, client_version,

            username, password, 
            ticket: RwLock::new(Token::zero()),
            http_client: http,
            default_character: CharacterId(0),

            own_characters: BiBTreeMap::new(),
            bookmarks: Vec::new(),
            friends: Default::default(),

            channel_data: DashMap::new(),
            character_data: DashMap::new(),
            admins: Default::default(),
            ignorelist: Default::default(),
            global_channels: DashSet::new(),

            sessions: RwLock::new(Vec::new()),
            send_channel: send,
            rcv_channel: AsyncMutex::new(rcv),


            event_listener,
        }
    }

    pub async fn init(&mut self) -> Result<(), ClientError> {
        let mut ticket = get_api_ticket(&self.http_client, &self.username, &self.password, true).await?;
        self.ticket.write().update(ticket.ticket);
        self.default_character = ticket.default_character;

        self.own_characters = ticket.characters.drain().collect();
        self.bookmarks = ticket.bookmarks;
        // This function should probably return this value and clone-map it into friends instead
        // Friend data is repopulated when a session is started.
        self.friends = RwLock::new(ticket.friends.drain(..).map(|f|f.dest).collect());

        Ok(())
    }

    pub async fn refresh(&self) -> Result<(), ClientError> {
        let ticket = get_api_ticket(&self.http_client, &self.username, &self.password, false).await?;
        self.ticket.write().update(ticket.ticket);
        Ok(())
    }

    pub async fn refresh_fast(&self) -> Result<(), ClientError> {
        // Optimistically refresh if the token is more than 20 minutes old
        // Supposedly it lasts 30 minutes but I don't trust these devs and their crap API
        if self.ticket.read().expired() {
            self.refresh().await?;
        }
        Ok(())
    }

    pub async fn connect(&self, character: Character) -> Result<JoinHandle<()>, ClientError> {
        self.refresh_fast().await?;
        let (mut socket, _) = connect_async_tls_with_config("wss://chat.f-list.net/chat2", None, None).await?;
        
        let ticket = self.ticket.read().ticket.clone();
        
        socket.send(Message::Text(prepare_command(&ClientCommand::Identify { 
            method: IdentifyMethod::Ticket, 
            account: self.username.clone(), 
            ticket, 
            character: character.clone(), 
            client_name: self.client_name.clone(), 
            client_version: self.client_version.clone() 
        }))).await?;

        let (write, read) = socket.split();

        let session = Arc::new(Session {
            character,
            write: AsyncMutex::new(write),
            channels: BTreeSet::new(),
            pms: DashMap::new(),
        });
        self.sessions.write().push(session.clone());

        // Oh, and something will need to listen to these...
        let chan = self.send_channel.clone();
        Ok(tokio::spawn(read.for_each(move |res| {
            // We don't want this to happen concurrently, because the events need to arrive in order
            // But they only need to arrive in order for any given connection.
            // Connections will end up interleaved in the channel consumer.
            let chan = chan.clone();
            let session = session.clone();
            async move {
                match res {
                    Err(err) => {eprintln!("Error when reading from session: {err:?}");},
                    Ok(ok) => {
                        match ok.to_text() {
                            Err(err) => {eprintln!("Message frame is not text: {err:?}");},
                            Ok(text) => {
                                let command = parse_command(text);
                                chan.send(Event {
                                    command,
                                    session
                                }).await.expect("Failed to send command through Tokio mpsc channel");
                            }
                        };
                    }
                };
            }
        })))
    }

    async fn dispatch(&self, event: Event) {
        match event.command {
            ServerCommand::Hello {message} => self.event_listener.hello(event.session, &message),
            ServerCommand::Error {number, message} => self.event_listener.raw_error(event.session, number, &message),
            ServerCommand::Connected { count } => self.event_listener.connected(event.session, count),

            // Messages
            ServerCommand::Broadcast { message, character } => self.event_listener.message(event.session, &MessageSource::Character(character), &MessageTarget::Broadcast, &message),
            ServerCommand::Message { character, message, channel } => self.event_listener.message(event.session, &MessageSource::Character(character), &MessageTarget::Channel(channel), &message),
            ServerCommand::PrivateMessage { character, message } => self.event_listener.message(event.session, &MessageSource::Character(character.clone()), &MessageTarget::PrivateMessage(character), &message),
            ServerCommand::SystemMessage { message, channel } => self.event_listener.message(event.session, &MessageSource::System, &MessageTarget::Channel(channel), &message),

            // State updates (Channels)
            ServerCommand::ChannelData { mut users, channel, mode } => {
                // CharacterIdentity is just Character, but structurally different because F-Chat.
                // Turn it into Character when storing it.
                // Maybe later can transmute if I promise they have the same memory layout (transparent String)
                let mut entry = self.channel_data.entry(channel).or_default();
                entry.channel_mode = mode;
                entry.members = users.drain(..).map(|v|v.identity).collect();
            },
            ServerCommand::ChannelDescription { channel, description } => {
                let mut entry = self.channel_data.entry(channel).or_default();
                entry.description = description;
            }
            ServerCommand::ChannelMode { mode, channel } => {
                let mut entry = self.channel_data.entry(channel).or_default();
                entry.channel_mode = mode;
            }
            ServerCommand::JoinedChannel { channel, character, title } => {
                let mut entry = self.channel_data.entry(channel).or_default();
                entry.title = title;
                entry.members.insert(character.identity);
            }
            ServerCommand::LeftChannel { channel, character } => {
                let mut entry = self.channel_data.entry(channel).or_default();
                entry.members.remove(&character);
            }
            // If you think about it, typing is just a state update for a PM channel. Sort of.
            ServerCommand::Typing { character, status } => {
                event.session.pms.insert(character.clone(), status);
                self.event_listener.typing(event.session, character, status);
            }

            // State updates (Characters)
            /*
            ServerCommand::ProfileData { response_type, message, key, value } => {
                // There's two ways to handle this:
                // - Statefully, where the client tracks the start/end and issues a profile update at the end
                // - Statelessly, where the client updates every time and emits an update event at the end
                // In this case, stateless simply makes the most sense.
                // However, because the profiles are keyed with strong keys rather than strings, it'll need matching.
                // There are also potential race conditions for profile updates, but I'm not sure that I actually care.
                match response_type {
                    ProfileDataPart::Start => {},
                    ProfileDataPart::Info => {},
                    ProfileDataPart::Select => {},
                    ProfileDataPart::End => {}
                }
            }
            */
            ServerCommand::ListOnline { mut characters } => {
                // This command is *sinful* and floods with every character's info. All of them.
                for character in characters.drain(..) {
                    self.character_data.insert(character.0, CharacterData {
                        gender: character.1,
                        status: character.2,
                        status_message: character.3,
                    });
                }
            },
            ServerCommand::NewConnection { status, gender, identity } => {
                self.character_data.insert(identity, CharacterData {
                    gender, status, status_message: "".to_owned()
                });
            },
            ServerCommand::Offline { character } => {
                // This could instead just *remove* the character.
                // But that would get rid of the latent gender information
                // Most people appear to expect to see None gender for Offline chars, but we can retain it.
                self.character_data.entry(character).and_modify(|v| {
                    v.status = Status::Offline;
                    v.status_message = "".to_owned();
                });
            },
            ServerCommand::Status { status, character, statusmsg } => {
                self.character_data.entry(character).and_modify(|v| {
                    v.status = status;
                    v.status_message = statusmsg;
                });
            },

            // State updates (global again)
            ServerCommand::Friends { characters } => {
                // This is just cheap to do, although arguably potentially incorrect.
                *self.friends.write() = characters;
            },
            ServerCommand::GlobalOps { ops } => {
                *self.admins.write() = ops;
            },
            ServerCommand::Ignore { action, characters, character } => {
                match action {
                    IgnoreAction::Init | IgnoreAction::List => *self.ignorelist.write() = characters,
                    IgnoreAction::Add => self.ignorelist.write().push(character),
                    IgnoreAction::Delete => self.ignorelist.write().retain(|v| *v!=character), 
                    _ => println!("Unhandled ignore action {action:?} -- {characters:?} -- {character:?}")
                }
            },
            ServerCommand::GlobalChannels { mut channels } => {
                for channel in channels.drain(..) {
                    self.global_channels.insert(channel);
                }
            },

            // Ping
            ServerCommand::Ping => {
                event.session.send(ClientCommand::Pong).await.expect("Failed to Pong!");
                self.event_listener.ping(event.session);
            }

            _ => eprintln!("Unhandled server command {event:?}")
        }
    }

    pub async fn start(&self) {
        let mut chan = self.rcv_channel.lock().await; // Lock it away forever. Sorry kid - Mine now.
        // using an async Mutex only because this holds the lock across an await boundary
        while let Some(event) = chan.recv().await {
            self.dispatch(event).await;
        }
    }
}

pub trait EventListener {
    fn raw_error(&self, ctx: Arc<Session>, id: i32, message: &str) {
        // Map the ID to an appropriate known error type. Use enums.
        let err: ProtocolError = id.into();
        if err.is_fatal() {
            panic!("Fatal error: {err:?}")
        }
        if err.has_message() {
            eprintln!("Error {err:?} -- {message}")
        } else {
            eprintln!("Error {err:?}")
        }
    }

    // Maybe unimplemented!() for these?
    fn hello(&self, ctx: Arc<Session>, message: &str) {} 
    fn connected(&self, ctx: Arc<Session>, count: u32) {}
    fn ping(&self, ctx: Arc<Session>) {}
    fn message(&self, ctx: Arc<Session>, source: &MessageSource, target: &MessageTarget, message: &str) {}
    fn typing(&self, ctx: Arc<Session>, character: Character, status: TypingStatus) {}
    fn error() {}
}

// Oh, and let's introduce a variety of non-protocol abstractions, to unify the client abstraction.
#[derive(Debug)]
pub enum MessageTarget {
    Broadcast,
    Channel(Channel),
    PrivateMessage(Character), 
}

#[derive(Debug)]
pub enum MessageSource {
    System,
    Character(Character)
}