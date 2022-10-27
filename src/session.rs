use std::sync::{Arc, atomic::AtomicI32};

use dashmap::{DashMap, DashSet}; use thiserror::Error;
// Optionally switch to BTree and manually manage R/W sync
use tokio::{sync::{Mutex as AsyncMutex, mpsc::{Sender, error::SendError}}, task::JoinHandle};
use futures_util::{stream::{SplitSink, SplitStream}, SinkExt, StreamExt, TryStreamExt};
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream, tungstenite::{Message, error::ProtocolError as WebsocketError}, connect_async_tls_with_config};
use tokio::net::TcpStream;

use crate::{data::{Character, TypingStatus, Channel}, protocol::{ServerCommand, ClientCommand, IdentifyMethod, prepare_command, parse_command, Variable, ProtocolError}};

#[derive(Debug, Default, Clone)]
pub struct Variables {
    pub chat_max: u32,
    pub priv_max: u32,
    pub ad_max: u32,
    pub chat_cooldown: f32,
    pub ad_cooldown: f32,
    pub status_cooldown: f32,
    pub icon_blacklist: Vec<Channel>, // I'm not sure that this is actually session-bound
    // Actually I'm not sure that any of these are session-bound.
}

#[derive(Debug)]
pub enum SessionEvent {
    Reconnect, // Asking the session-manager to reconnect the session.
    Disconnected(ProtocolError), // Fatal* disconnects caused by server
    Command(ServerCommand),
    Error(SessionError)
}

#[derive(Debug)]
pub struct Event {
    session: Arc<Session>,
    event: SessionEvent
}

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;
type StreamWriter = AsyncMutex<SplitSink<Socket, Message>>;

#[derive(Debug)]
pub struct Session {
    pub character: Character,
    pub channels: DashSet<Channel>,
    pub private_messages: DashMap<Character, TypingStatus>,
    pub variables: Variables, // I'm not sure that these are actually session-bound
    pub last_err: AtomicI32,

    write: StreamWriter,
    event_channel: Sender<Event>
}

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Error from Websocket (Tungstenite)")]
    WebsocketError(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Unknown failure in connection stage -- Typically protocol-related")]
    MiscConnectionFailure,
    #[error("Unknown or unexpected protocol message: {0}")]
    UnexpectedProtocolMessage(String),
    #[error("VAR command has arrived out of sync")]
    LateVarCommand,
    #[error("IDN command has arrived out of sync")]
    LateIdentifyCommand
}

pub type SessionResult<T> = Result<T, SessionError>;

impl Session {
    const WS_URL: &'static str = "wss://chat.f-list.net/chat2";

    pub async fn connect(account: String, ticket: String, client_name: String, client_version: String, character: Character, event_channel: Sender<Event>) -> SessionResult<Arc<Self>> {
        let mut socket = Session::connect_internal(account, ticket, client_name, client_version, character).await?;
        let (variables, next) = Session::read_variables(&mut socket).await?;
        let (write, read) = socket.split();

        let session = Arc::new(Session {
            character,
            channels: DashSet::new(),
            private_messages: DashMap::new(),
            variables,
            last_err: AtomicI32::new(ProtocolError::Other as i32),

            write: AsyncMutex::new(write),
            event_channel
        });
        Session::start_event_loop(session.clone(), read)?;

        Ok(session)
    }

    // Produces a new Session based off of the current Session.
    pub async fn reconnect(&self, account: String, ticket: String, client_name: String, client_version: String) -> SessionResult<Arc<Self>> {
        let mut socket = Session::connect_internal(account, ticket, client_name, client_version, self.character).await?;
        let (variables, next) = Session::read_variables(&mut socket).await?;
        let (write, read) = socket.split();

        let session = Arc::new(Session {
            character: self.character,
            channels: DashSet::new(),
            private_messages: DashMap::new(),
            variables,
            last_err: AtomicI32::new(ProtocolError::Other as i32),
            
            write: AsyncMutex::new(write),
            event_channel: self.event_channel.clone(),
        });
        Session::start_event_loop(session.clone(), read)?;

        // Now try to re-join all of the old channels.
        let mut write = session.write.lock().await;
        for channel in self.channels.iter() {
            write.feed(Message::Text(prepare_command(&ClientCommand::JoinChannel { channel: channel.to_owned() }))).await?
        }
        write.flush().await;
        drop(write); // If I don't drop here, it complains that the guard still exists when I return session.

        Ok(session)
    }

    // Sometimes, the existing session needs to be reconnected.
    // Because this uses the same logic as connect, this is abstracted.
    async fn connect_internal(account: String, ticket: String, client_name: String, client_version: String, character: Character) -> SessionResult<Socket> {
        // Establish the connection
        let (mut socket, _) = connect_async_tls_with_config(Self::WS_URL, None, None).await?;
        
        // Identify (IDN)
        socket.send(Message::Text(prepare_command(&ClientCommand::Identify { 
            method: IdentifyMethod::Ticket, 
            account, 
            ticket, 
            character, 
            client_name, 
            client_version 
        }))).await?;

        // Wait for IDN response or blow up (protocol error)
        // Messages sent are -always- Text
        // Never Ping, Close, etc. Server does not follow recommendations for closing connections.
        if let Some(Message::Text(message)) = socket.try_next().await? {
            if let ServerCommand::IdentifySuccess { character: character_id } = parse_command(&message) {
                assert_eq!(character, character_id);
                Ok(socket)
            } else {
                Err(SessionError::UnexpectedProtocolMessage(message))
            }
        } else {
            Err(SessionError::MiscConnectionFailure)
        }
    }

    async fn emit_event(session: &Arc<Session>, event: SessionEvent) -> Result<(), SendError<Event>> {
        session.event_channel.send(Event {
            session: session.clone(), event
        }).await
    }

    // Reads VAR from a socket until there's no more VAR, and yields the next command (should be HLO)
    async fn read_variables(socket: &mut Socket) -> SessionResult<(Variables, ServerCommand)> {
        let mut vars: Variables = Default::default();
        loop {
            if let Some(Message::Text(message)) = socket.try_next().await? {
                match parse_command(&message) {
                    ServerCommand::Variable(var) => match var {
                        Variable::ChatMax(v) => vars.chat_max = v,
                        Variable::PrivMax(v) => vars.priv_max = v,
                        Variable::AdMax(v) => vars.ad_max = v,
                        Variable::AdCooldown(v) => vars.ad_cooldown = v,
                        Variable::ChatCooldown(v) => vars.chat_cooldown = v,
                        Variable::StatusCooldown(v) => vars.status_cooldown = v,
                        Variable::IconBlacklist(v) => vars.icon_blacklist = v,
                        other => eprintln!("Unhandled var {other:?}")
                    },
                    other => {
                        return Ok((vars, other))
                    }
                }
            } else {
                return Err(SessionError::MiscConnectionFailure);
            }
        }
    }

    fn start_event_loop(session: Arc<Session>, read: SplitStream<Socket>) -> SessionResult<JoinHandle<()>> {
        Ok(tokio::spawn(read.for_each(move |res| async move {
            // We don't want this to happen concurrently, because the events need to arrive in order
            // But they only need to arrive in order for any given connection.
            // Connections will end up interleaved in the channel consumer.
            match res {
                Err(err) => match err {
                    tokio_tungstenite::tungstenite::Error::Protocol(err) => match err {
                        WebsocketError::ResetWithoutClosingHandshake => {
                            // The server has closed the connection. It never sends close frames.
                            // Check for the most recent ERR type, and if it's fatal.
                            let last_err = ProtocolError::from(session.last_err.load(std::sync::atomic::Ordering::SeqCst));
                            if last_err.is_fatal() {
                                Session::emit_event(&session, SessionEvent::Disconnected(last_err)).await.expect("Failed to send event through event channel (disconnect)")
                            } else {
                                Session::emit_event(&session, SessionEvent::Reconnect).await.expect("Failed to send event through event channel (reconnect)")
                            }
                        },
                        WebsocketError::ReceivedAfterClosing => eprintln!("Close frames are not respected by F-Chat"),
                        other => panic!("Unexpected (protocol) error from Tungstenite: {other:?}")
                    },
                    other => panic!("Unexpected error from Tungstenite: {other:?}")
                },
                Ok(Message::Text(command)) => {
                    let command = parse_command(&command);
                    // Handle the command and decide if we should forward it to the event channel
                    match Session::handle_command(&session, &command).await {
                        Ok(true) => Session::emit_event(&session, SessionEvent::Command(command)).await.expect("Failed to send event through event channel (command)"),
                        Err(err) => Session::emit_event(&session, SessionEvent::Error(err)).await.expect("Failed to send event through event channel (error)"),
                        Ok(false) => {} // Do nothing; there was no error, and we're not fowarding the command.
                    }
                },
                Ok(other) => {
                    eprintln!("Unexpected frame from F-Chat: {other:?}")
                }
            }
        })))
    }

    async fn handle_command(session: &Arc<Session>, command: &ServerCommand) -> SessionResult<bool> {
        match command {
            ServerCommand::Ping => session.send(ClientCommand::Pong).await.map(|_| false),
            ServerCommand::Hello { .. } => Ok(false), 
            ServerCommand::Connected { .. } => Ok(true), // Because Connected is sent after Hello, it's a better "ready" event

            ServerCommand::Error { number, message } => {
                session.last_err.store(*number, std::sync::atomic::Ordering::Relaxed);
                Ok(true)
            },

            ServerCommand::JoinedChannel { channel, character, title } => {
                if *character == session.character {
                    // If it was this session, update the joined-channels list.
                    session.channels.insert(*channel);
                }
                Ok(true)
            },
            ServerCommand::LeftChannel { channel, character } => {
                if *character == session.character {
                    // As above, so below.
                    session.channels.remove(channel);
                }
                Ok(true)
            },

            ServerCommand::Typing { character, status } => {
                if let Some(old) = session.private_messages.insert(*character, *status) {
                    Ok(old == *status)
                } else {
                    Ok(true)
                }
            },

            ServerCommand::IdentifySuccess { .. } => Err(SessionError::LateIdentifyCommand),
            ServerCommand::Variable(_) => Err(SessionError::LateVarCommand),

            _ => Ok(true)
        }
    }

    pub async fn send(&self, command: ClientCommand) -> SessionResult<()> {
        Ok(self.write.lock().await.send(Message::Text(prepare_command(&command))).await?)
    }

    pub async fn send_message(&self, target: MessageTarget, message: String) -> SessionResult<()> {
        match target {
            MessageTarget::Broadcast => self.send(ClientCommand::Broadcast { message }).await?,
            MessageTarget::Channel(channel) => self.send(ClientCommand::Message { channel, message }).await?,
            MessageTarget::PrivateMessage(recipient) => {
                let (ra, rb) = join!(
                    self.send(ClientCommand::PrivateMessage { recipient: recipient.clone(), message }),
                    self.send(ClientCommand::Typing {
                        character: recipient,
                        status: TypingStatus::Clear
                    })
                );
                return ra.and(rb);
            }
        };
        Ok(())
    }

    pub async fn send_dice(&self, target: MessageTarget, dice: String) -> SessionResult<()> {
        self.send(match target {
            MessageTarget::Broadcast => panic!("You can't broadcast dice!"), // Upstream invalid state. Implementor logic error.
            MessageTarget::Channel(channel) => ClientCommand::Roll { target: Target::Channel { channel }, dice },
            MessageTarget::PrivateMessage(recipient) => ClientCommand::Roll { target: Target::Character { recipient }, dice }
        }).await
    }

    pub async fn join_channel(&self, channel: Channel) -> SessionResult<()> {
        self.send(ClientCommand::JoinChannel { channel }).await
    }
}