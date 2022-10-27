// Cache trait for the client. 
use std::borrow::Cow;

use serde::{Serialize, Deserialize};

use crate::{data::{Character, Channel, ChannelMode, Gender, Status, FriendRelation}, util::timestamp::Timestamp};

pub trait Cache {
    type Error: std::error::Error;

    fn insert_message(&self, message: MessageData) -> Result<(), Self::Error>;
    fn insert_channel(&self, channel: Cow<Channel>, data: PartialChannelData, members: Cow<Vec<Character>>);

    fn add_channel_member(&self, channel: Cow<Channel>, member: Character) -> Result<bool, Self::Error>;
    fn remove_channel_member(&self, channel: Cow<Channel>, member: Character) -> Result<bool, Self::Error>;

    fn update_channel(&self, channel: Cow<Channel>, data: PartialChannelData) -> Result<bool, Self::Error>;
    fn update_character(&self, character: Cow<Character>, data: PartialUserData) -> Result<bool, Self::Error>;
    
    fn set_friends(&self, friends: Cow<Vec<FriendRelation>>) -> Result<bool, Self::Error>;
    fn set_bookmarks(&self, bookmarks: Cow<Vec<Character>>) -> Result<bool, Self::Error>;
    fn set_channel_members(&self, channel: Cow<Channel>, members: Cow<Vec<Character>>) -> Result<bool, Self::Error>;
    
    fn get_channel(&self, channel: &Channel) -> Result<Option<ChannelData>, Self::Error>;
    fn get_channels(&self) -> Result<Vec<ChannelData>, Self::Error>;
    fn get_character(&self, character: &Character) -> Result<Option<CharacterData>, Self::Error>;
    fn get_characters(&self) -> Result<Vec<CharacterData>, Self::Error>;
    fn get_messages(&self, source: &MessageChannel, since: Option<Timestamp>, limit: Option<u32>) -> Result<Vec<MessageData>, Self::Error>;
    fn get_friend_relations(&self) -> Result<Vec<FriendRelation>, Self::Error>;
    fn get_friends(&self) -> Result<Vec<Character>, Self::Error> {
        self.get_friend_relations().map(|mut v|v.drain(..).map(|c|c.other_character).collect())
    }
    fn get_bookmarks(&self) -> Result<Vec<Character>, Self::Error>;
}

// Note RE: FriendRelation; go back and rename the fields to have own_character/other_character

#[derive(Serialize, Debug)]
pub struct PartialChannelData<'a> {
    mode: Option<ChannelMode>,
    title: Option<Cow<'a, String>>,
    description: Option<Cow<'a, String>>
}

#[derive(Serialize, Debug)]
pub struct PartialUserData<'a> {
    gender: Option<Gender>,
    status: Option<Status>,
    status_message: Option<Cow<'a, String>>
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ChannelData {
    channel: Channel,
    channel_mode: ChannelMode,
    members: Vec<Character>,
    description: String,
    title: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CharacterData {
    pub character: Character,
    pub gender: Gender,
    pub status: Status,
    pub status_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageData {
    pub content: MessageContent,
    pub character: Character,
    pub channel: MessageChannel
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MessageContent {
    Message {content: String},
    Emote {emote: String},
    Dice {rolls: Vec<String>, results: Vec<i32>, total: i32},
    Bottle {bottle: Character}
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(untagged)]
pub enum MessageChannel {
    PrivateMessage {session: Character, recipient: Character},
    Channel {channel: Channel},
    Console {console: Character}
}

#[derive(thiserror::Error, Debug)]
pub enum NoCacheError {} // Never construct any instances of this

#[derive(Debug)]
pub struct NoCache;
impl Cache for NoCache {
    // Basic no-op cache which always emits update events
    type Error = NoCacheError;

    fn insert_message(&self, message: MessageData) -> Result<(), Self::Error> {
        Ok(())
    }

    fn insert_channel(&self, channel: Cow<Channel>, data: PartialChannelData, members: Cow<Vec<Character>>) {
        todo!()
    }

    fn update_channel(&self, channel: Cow<Channel>, data: PartialChannelData) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn update_character(&self, character: Cow<Character>, data: PartialUserData) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn get_channel(&self, channel: &Channel) -> Result<Option<ChannelData>, Self::Error> {
        Ok(None)
    }

    fn get_channels(&self) -> Result<Vec<ChannelData>, Self::Error> {
        Ok(Vec::new())
    }

    fn get_character(&self, character: &Character) -> Result<Option<CharacterData>, Self::Error> {
        Ok(None)
    }

    fn get_characters(&self) -> Result<Vec<CharacterData>, Self::Error> {
        Ok(Vec::new())
    }

    fn add_channel_member(&self, channel: Cow<Channel>, member: Character) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn remove_channel_member(&self, channel: Cow<Channel>, member: Character) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn get_messages(&self, source: &MessageChannel, since: Option<Timestamp>, limit: Option<u32>) -> Result<Vec<MessageData>, Self::Error> {
        Ok(Vec::new())
    }

    fn set_friends(&self, friends: Cow<Vec<FriendRelation>>) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn set_bookmarks(&self, bookmarks: Cow<Vec<Character>>) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn set_channel_members(&self, channel: Cow<Channel>, members: Cow<Vec<Character>>) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn get_friend_relations(&self) -> Result<Vec<FriendRelation>, Self::Error> {
        Ok(Vec::new())
    }

    fn get_bookmarks(&self) -> Result<Vec<Character>, Self::Error> {
        Ok(Vec::new())
    }

    
}