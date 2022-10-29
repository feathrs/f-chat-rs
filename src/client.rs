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

pub use async_trait::async_trait;
use chrono::Utc;

use std::{time::{Instant, Duration}, sync::Arc, borrow::Cow};
use parking_lot::RwLock;
use thiserror::Error;

use reqwest::Client as ReqwestClient;
use tokio::sync::mpsc::{Sender, Receiver, channel};

use crate::{http_endpoints::{get_api_ticket, self}, data::{Character, Channel, Status, TypingStatus, MessageChannel, Message, MessageContent, FriendRelation}, protocol::*, cache::{Cache, NoCache, PartialChannelData, PartialUserData}, session::{Event, Session, SessionError}};

#[derive(Debug)]
pub struct Client<T: EventListener, C: Cache> {
    client_name: String,
    client_version: String,

    username: String,
    password: String,
    token: RwLock<Token>,
    http_client: ReqwestClient,
    pub default_character: Character,
    pub own_characters: Vec<Character>,

    pub cache: C,

    sessions: RwLock<Vec<Arc<Session>>>,
    send_channel: Sender<Event>,

    event_listener: T
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Error from HTTP Request")]
    RequestError(#[from] reqwest::Error),
    #[error("Error from Websocket (Tungstenite)")]
    WebsocketError(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Default character doesn't exist or is invalid")]
    NoDefaultCharacter,
    #[error("Error from Session implementation")]
    SessionError(#[from] crate::session::SessionError)
}
type ClientResult<T> = Result<T, ClientError>;

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
            last_updated: Instant::now() // Good luck.
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
impl Default for Token {
    fn default() -> Self {
        Self::zero()
    }
}

#[derive(Debug)]
pub struct ClientBuilder<E: EventListener, C: Cache> {
    client_version: String,
    client_name: String,
    events: E,
    cache: C
}

impl<E: EventListener> ClientBuilder<E, NoCache> {
    pub fn new(events: E) -> Self {
        ClientBuilder { 
            client_version: option_env!("CARGO_PKG_VERSION").unwrap_or("0.1").to_owned(), 
            client_name: "f-chat-rs".to_string(), 
            events, 
            cache: NoCache 
        }
    }
}

impl<E: EventListener + 'static, C: Cache + 'static> ClientBuilder<E, C> {
    pub fn with_cache<C2: Cache>(self, cache: C2) -> ClientBuilder<E, C2> {
        ClientBuilder {
            client_version: self.client_version,
            client_name: self.client_name,
            events: self.events,
            cache
        }
    }

    pub fn with_version(self, client_name: String, client_version: String) -> Self {
        ClientBuilder {
            client_name, client_version,
            ..self
        }
    }

    pub async fn init(self, username: String, password: String) -> ClientResult<(Client<E, C>, Receiver<Event>)> {
        let http = ReqwestClient::new();
        let (send, rcv) = channel(8);
        let ticket_init = get_api_ticket(&http, &username, &password, true).await?;
        let token = Token::new(ticket_init.ticket);
        
        let mut extra = ticket_init.extra.expect("Ticket init response did not send extra fields");
        let default_char = extra.default_character;
        let default_character = extra.characters.iter()
            .find_map(move |(char, id)| if id == &default_char {Some(*char)} else {None})
            .ok_or(ClientError::NoDefaultCharacter)?;
        let own_characters = extra.characters.drain().map(|(character, _)|character).collect();

        self.cache.set_bookmarks(extra.bookmarks.drain(..).map(|v|v.name).collect::<Vec<_>>().into()).unwrap();
        self.cache.set_friends(extra.friends.drain(..).map(|v|FriendRelation{own_character:v.dest, other_character:v.source}).collect::<Vec<_>>().into()).unwrap();

        let client = Client {
            client_name: self.client_name,
            client_version: self.client_version,
            username, password,
            token: RwLock::new(token),
            http_client: http,
            default_character,
            own_characters,
            cache: self.cache,
            sessions: Default::default(),
            send_channel: send,
            event_listener: self.events,
        };

        Ok((client, rcv))
    }
}

impl<T: EventListener, C: Cache> Client<T, C> {
    pub async fn start(&self, mut rcv: Receiver<Event>) {
        while let Some(event) = rcv.recv().await {
            self.dispatch(event).await;
        }
    }

    pub async fn refresh(&self) -> Result<(), ClientError> {
        let ticket = get_api_ticket(&self.http_client, &self.username, &self.password, false).await?;
        self.token.write().update(ticket.ticket);
        Ok(())
    }

    pub async fn refresh_fast(&self) -> Result<(), ClientError> {
        // Optimistically refresh if the token is more than 20 minutes old
        // Supposedly it lasts 30 minutes but I don't trust these devs and their crap API
        if self.token.read().expired() {
            self.refresh().await?;
        }
        Ok(())
    }

    pub async fn connect(&self, character: Character) -> ClientResult<()> {
        self.refresh().await?;
        let token = self.token.read().ticket.clone();
        let session = Session::connect(
            self.username.clone(),
            token, 
            self.client_name.clone(), 
            self.client_version.clone(), 
            character, 
            self.send_channel.clone()
        ).await?;

        // Add the new session to the list, to hold on to it.
        self.sessions.write().push(session);
        Ok(())
    }

    pub async fn sync_friends_bookmarks(&self) -> ClientResult<bool> {
        // Events are mostly emitted through the event-handler.
        self.refresh_fast().await?;
        let ticket = self.token.read().ticket.clone();
        let mut list = http_endpoints::get_friends_list(&self.http_client, &ticket, &self.username).await?.inner;
        let update_bookmarks = self.cache.set_bookmarks(list.bookmarks.into()).unwrap();
        let update_friends = self.cache.set_friends(
            Cow::from(list.friends.drain(..).map(|v|FriendRelation{own_character:v.dest, other_character:v.source}).collect::<Vec<_>>())
        ).unwrap();
        if update_bookmarks {self.event_listener.updated_bookmarks().await}
        if update_friends {self.event_listener.updated_friends().await}
        Ok(update_friends || update_bookmarks)
    }

    pub fn get_session(&self, session: &Character) -> Option<Arc<Session>> {
        self.sessions.read().iter().find(|this_session| this_session.character == *session).cloned()
    }

    pub fn get_sessions(&self) -> Vec<Arc<Session>> {
        self.sessions.read().clone()
    }

    fn drop_session(&self, session: &Character) {
        self.sessions.write().retain(|v|v.character != *session)
    }

    #[allow(unused_variables)]
    pub(crate) async fn dispatch(&self, event: Event) {
        match event.event {
            crate::session::SessionEvent::Reconnect => {
                // Reconnect the session; treat it as having disconnected
                self.refresh_fast().await.unwrap();
                let ticket = self.token.read().ticket.clone();
                let new_session = event.session.reconnect(
                    self.username.clone(), 
                    ticket, 
                    self.client_name.clone(), 
                    self.client_version.clone()
                ).await;
                self.drop_session(&event.session.character);
                match new_session {
                    Ok(session) => self.sessions.write().push(session),
                    Err(err) => self.event_listener.session_error(event.session, err).await,
                }
                self.event_listener.sessions_updated().await
            },
            crate::session::SessionEvent::Disconnected(err) => {
                self.drop_session(&event.session.character);
                self.event_listener.session_disconnected(event.session, err).await;
                self.event_listener.sessions_updated().await;
            },
            crate::session::SessionEvent::Command(command) => {
                self.event_listener.raw_command(event.session.clone(), &command).await;
                match command {
                    ServerCommand::GlobalOps { ops } => if self.cache.set_global_ops(ops.into()).unwrap() {self.event_listener.updated_global_ops().await},
                    ServerCommand::GlobalOpped { character } => if self.cache.add_global_op(Cow::Owned(character)).unwrap() {self.event_listener.updated_global_ops().await},
                    ServerCommand::GlobalDeopped { character } => if self.cache.remove_global_op(Cow::Owned(character)).unwrap() {self.event_listener.updated_global_ops().await},

                    ServerCommand::Banned { operator, channel, character } => todo!("(Banned event)"), // Need a moderation abstraction
                    ServerCommand::Kicked { operator, channel, character } => todo!("(Kick event)"), // Each event has channel, character, operator
                    ServerCommand::Timeout { channel, character, length, operator } => todo!("(Timeout event)"), // No examples of use though.

                    ServerCommand::Broadcast { message, character } => self.event_listener.broadcast(character, message).await,
                    ServerCommand::ChannelDescription { channel, description } => {
                        if self.cache.update_channel(Cow::Borrowed(&channel), PartialChannelData {
                            description: Some(description.into()),
                            ..Default::default()
                        }).unwrap() {
                            self.event_listener.updated_channel(channel).await
                        }
                    },
                    ServerCommand::GlobalChannels { mut channels } => {
                        for channel in channels.iter() {
                            if self.cache.update_channel(Cow::Borrowed(&channel.name), PartialChannelData {
                                title: Some(Cow::from(channel.name.0.as_ref())),
                                mode: Some(channel.mode),
                                ..Default::default()
                            }).unwrap() {
                                self.event_listener.updated_channel(channel.name).await
                            }
                        }
                        if self.cache.set_global_channels(Cow::Owned(channels.drain(..).map(|v|(v.name, v.characters)).collect())).unwrap() {
                            self.event_listener.updated_channel_lists().await
                        }
                    },
                    ServerCommand::Invited { sender, title, name } => {
                        if self.cache.update_channel(Cow::Borrowed(&name), PartialChannelData {
                            title: Some(title.into()),
                            ..Default::default()
                        }).unwrap() {
                            self.event_listener.updated_channel(name).await
                        }
                        self.event_listener.invited(event.session, name, sender).await
                    },
                    
                    ServerCommand::Opped { character, channel } => if self.cache.add_channel_op(Cow::Borrowed(&channel), Cow::Owned(character)).unwrap() {self.event_listener.updated_channel(channel).await},
                    ServerCommand::Ops { channel, oplist } => if self.cache.set_channel_ops(Cow::Borrowed(&channel), Cow::Owned(oplist)).unwrap() {self.event_listener.updated_channel(channel).await},
                    ServerCommand::Connected { .. } => self.event_listener.ready(event.session).await,
                    ServerCommand::Deopped { character, channel } => if self.cache.remove_channel_op(Cow::Borrowed(&channel), Cow::Owned(character)).unwrap() {self.event_listener.updated_channel(channel).await},
                    ServerCommand::SetOwner { character, channel } => todo!(), // It is unclear how this information is conveyed otherwise.
                    ServerCommand::Error { number, message } => self.event_listener.error(event.session, number.into(), message).await,
                    ServerCommand::Search { characters, kinks } => todo!("FKS -- Should never reach client"), // Wrap up into session 
                    ServerCommand::Offline { character } => {
                        if self.cache.update_character(Cow::Borrowed(&character), PartialUserData {
                            status: Some(Status::Offline),
                            ..Default::default()
                        }).unwrap() {
                            self.event_listener.updated_character(character).await
                        }
                    },
                    ServerCommand::Hello { .. } => panic!("HLO -- Should never reach client"), // Sunk by session impl
                    ServerCommand::ChannelData { users, channel, mode } => {
                        if self.cache.insert_channel(Cow::Borrowed(&channel), PartialChannelData {
                            mode: Some(mode),
                            ..Default::default()
                        }, Cow::Owned(users)).unwrap() {
                            self.event_listener.updated_channel(channel).await
                        }
                    },
                    ServerCommand::IdentifySuccess { .. } => panic!("IDN -- Should never reach client"), // Sunk by session impl
                    ServerCommand::JoinedChannel { channel, character, title } => {
                        if self.cache.update_channel(Cow::Borrowed(&channel), PartialChannelData {
                            title: Some(title.into()),
                            ..Default::default()
                        }).unwrap() || self.cache.add_channel_member(Cow::Borrowed(&channel), character).unwrap() {
                            self.event_listener.updated_channel(channel).await
                        }
                        if event.session.character == character {
                            self.event_listener.updated_session_channels(event.session).await
                        }
                    },
                    ServerCommand::Kinks { .. } => eprintln!("Received KID from server -- Use HTTP/JSON endpoint instead"),
                    ServerCommand::LeftChannel { channel, character } => {
                        if self.cache.remove_channel_member(Cow::Borrowed(&channel), character).unwrap() {
                            self.event_listener.updated_channel(channel).await
                        }
                        if event.session.character == character {
                            self.event_listener.updated_session_channels(event.session).await
                        }
                    },
                    ServerCommand::ListOnline { mut characters } => {
                        for character in characters.drain(..) {
                            if self.cache.update_character(Cow::Borrowed(&character.0), PartialUserData {
                                gender: Some(character.1),
                                status: Some(character.2),
                                status_message: Some(character.3.into()),
                            }).unwrap() {
                                self.event_listener.updated_character(character.0).await
                            }
                        }
                    },
                    ServerCommand::NewConnection { status, gender, identity } => {
                        if self.cache.update_character(Cow::Borrowed(&identity), PartialUserData {
                            status: Some(status),
                            gender: Some(gender),
                            ..Default::default()
                        }).unwrap() {
                            self.event_listener.updated_character(identity).await
                        }
                    },
                    ServerCommand::Ignore { action, characters, character } => eprintln!("Missing handler for IGN"), // Should track
                    ServerCommand::Friends { .. } => {}, // We ignore this because it's bad data.
                    ServerCommand::Channels { mut channels } => {
                        for channel in channels.iter() {
                            if self.cache.update_channel(Cow::Borrowed(&channel.name), PartialChannelData {
                                title: Some(Cow::Borrowed(&channel.title)),
                                ..Default::default()
                            }).unwrap() {
                                self.event_listener.updated_channel(channel.name).await
                            }
                        }
                        if self.cache.set_unofficial_channels(Cow::Owned(channels.drain(..).map(|v|(v.name, v.characters)).collect())).unwrap() {
                            self.event_listener.updated_channel_lists().await
                        }
                    },
                    ServerCommand::Ping => panic!("PIN -- Should never reach client"), // Sunk by session impl
                    ServerCommand::ProfileData { .. } => eprintln!("Received PRD from server -- Use HTTP/JSON endpoint instead"),
                    ServerCommand::PrivateMessage { character, message } => {
                        let source = MessageChannel::PrivateMessage(event.session.character, character);
                        let content = MessageContent::Message(message.clone());
                        if self.cache.insert_message(source, Message {
                            timestamp: Utc::now(),
                            character,
                            content: content.clone(), // TODO: Ouch...
                        }).unwrap() {
                            self.event_listener.message(event.session, source, character, content).await
                        }
                    },
                    ServerCommand::Message { character, message, channel } => {
                        let source = MessageChannel::Channel(channel);
                        let content = MessageContent::Message(message.clone());
                        if self.cache.insert_message(source, Message {
                            timestamp: Utc::now(),
                            character,
                            content: content.clone(),
                        }).unwrap() {
                            self.event_listener.message(event.session, source, character, content).await
                        }
                    },
                    ServerCommand::Ad { character, message, channel } => {
                        if self.cache.insert_ad(Cow::Borrowed(&channel), Cow::Borrowed(&character), Cow::Borrowed(&message)).unwrap() {
                            self.event_listener.ad(channel, character, message).await;
                        }
                    },
                    ServerCommand::Roll { target, results, response_type, rolls, character, endresult, message } => {
                        // I hate this command signature with a passion fruit.
                        let source = match target {
                            Target::Channel { channel } => MessageChannel::Channel(channel),
                            Target::Character { recipient } => MessageChannel::PrivateMessage(recipient, character),
                        };
                        let content = MessageContent::Roll(rolls, results, endresult);
                        if self.cache.insert_message(source, Message {
                            timestamp: Utc::now(),
                            character,
                            content: content.clone(),
                        }).unwrap() {
                            self.event_listener.message(event.session, source, character, content).await
                        }
                    },
                    ServerCommand::ChannelMode { mode, channel } => {
                        if self.cache.update_channel(Cow::Borrowed(&channel), PartialChannelData {
                            mode: Some(mode),
                            ..Default::default()
                        }).unwrap() {
                            self.event_listener.updated_channel(channel).await
                        }
                    },
                    ServerCommand::BridgeEvent { response_type, name } => {
                        match response_type {
                            BridgeEvent::BookmarkAdd => {
                                if self.cache.add_bookmark(Cow::Owned(name)).unwrap() {
                                    self.event_listener.updated_bookmarks().await
                                }
                            },
                            BridgeEvent::BookmarkRemove => {
                                if self.cache.remove_bookmark(Cow::Owned(name)).unwrap() {
                                    self.event_listener.updated_bookmarks().await
                                }
                            },
                            BridgeEvent::FriendAdd | BridgeEvent::FriendRemove => {
                                // Both FriendAdd and FriendRemove don't include the full relation data,
                                // So we sync the friend list via the HTTP/JSON endpoint.
                                self.sync_friends_bookmarks().await.unwrap();
                            },
                            BridgeEvent::FriendRequest => {
                                eprintln!("Not handling RTB FriendRequest");
                            }
                        }
                    },
                    ServerCommand::Report { action, moderator, character, timestamp, callid, report, logid } => todo!(),
                    ServerCommand::Status { status, character, statusmsg } => {
                        if self.cache.update_character(Cow::Borrowed(&character), PartialUserData {
                            status: Some(status),
                            status_message: Some(statusmsg.into()),
                            ..Default::default()
                        }).unwrap() {
                            self.event_listener.updated_character(character).await
                        }
                    },
                    ServerCommand::SystemMessage { message, channel } => {
                        // May need to look into parsing system messages.
                        self.event_listener.system_message(event.session, channel, message).await
                    },
                    ServerCommand::Typing { character, status } => {
                        self.event_listener.typing(event.session, character, status).await
                    },
                    ServerCommand::Uptime { .. } => eprintln!("Not handling UPT"),
                    ServerCommand::Variable(_) => panic!("VAR -- Should never reach client"), // Sunk by Session impl
                }
            },
            crate::session::SessionEvent::Error(err) => {
                self.event_listener.session_error(event.session, err).await;
            },
        }
    }
}

#[async_trait]
#[allow(unused_variables)]
pub trait EventListener: std::marker::Sync + Sized + std::marker::Send {
    async fn raw_command(&self, ctx: Arc<Session>, command: &ServerCommand) {}

    async fn session_error(&self, ctx: Arc<Session>, error: SessionError) {}
    async fn sessions_updated(&self) {}
    async fn session_disconnected(&self, ctx: Arc<Session>, error: ProtocolError) {}
    async fn ready(&self, ctx: Arc<Session>) {}

    async fn broadcast(&self, character: Character, message: String) {}
    async fn invited(&self, ctx: Arc<Session>, channel: Channel, sender: Character) {}
    async fn ad(&self, channel: Channel, character: Character, ad: String) {}
    async fn system_message(&self, ctx: Arc<Session>, channel: Channel, message: String) {}
    async fn message(&self, ctx: Arc<Session>, channel: MessageChannel, character: Character, message: MessageContent) {}
    async fn typing(&self, ctx: Arc<Session>, character: Character, status: TypingStatus) {}

    async fn updated_friends(&self) {} // No need to send anything optimistically; end user can read off client
    async fn updated_bookmarks(&self) {} // Ditto for bookmarks, although I'm unsure how it behaves...
    async fn updated_channel(&self, channel: Channel) {} // Don't send the new data, because we don't track old data.
    async fn updated_character(&self, user: Character) {} 
    async fn updated_global_ops(&self) {}
    async fn updated_channel_lists(&self) {}
    async fn updated_session_channels(&self, session: Arc<Session>) {}

    async fn error(&self, ctx: Arc<Session>, err: ProtocolError, message: String) {
        // Map the ID to an appropriate known error type. Use enums.
        if err.is_fatal() {
            panic!("Fatal error: {err:?}")
        }
        if err.has_message() {
            eprintln!("Error {err:?} -- {message}")
        } else {
            eprintln!("Error {err:?}")
        }
    }
}