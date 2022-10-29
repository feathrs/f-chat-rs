// Cache trait for the client. 
use std::borrow::Cow;

use serde::Serialize;

use crate::{data::{Character, Channel, ChannelMode, Gender, Status, FriendRelation, MessageChannel, Message, ChannelData, CharacterData}, util::timestamp::Timestamp};

pub trait Cache: std::marker::Sync + Sized + std::marker::Send {
    type Error: std::error::Error;

    fn insert_message(&self, source: MessageChannel, message: Message) -> Result<bool, Self::Error>;
    fn insert_channel(&self, channel: Cow<Channel>, data: PartialChannelData, members: Cow<Vec<Character>>) -> Result<bool, Self::Error>;

    fn add_channel_member(&self, channel: Cow<Channel>, member: Character) -> Result<bool, Self::Error>;
    fn remove_channel_member(&self, channel: Cow<Channel>, member: Character) -> Result<bool, Self::Error>;

    fn add_bookmark(&self, character: Cow<Character>) -> Result<bool, Self::Error>;
    fn remove_bookmark(&self, character: Cow<Character>) -> Result<bool, Self::Error>;

    fn update_channel(&self, channel: Cow<Channel>, data: PartialChannelData) -> Result<bool, Self::Error>;
    fn update_character(&self, character: Cow<Character>, data: PartialUserData) -> Result<bool, Self::Error>;
    
    fn set_friends(&self, friends: Cow<Vec<FriendRelation>>) -> Result<bool, Self::Error>;
    fn set_bookmarks(&self, bookmarks: Cow<Vec<Character>>) -> Result<bool, Self::Error>;
    fn set_channel_members(&self, channel: Cow<Channel>, members: Cow<Vec<Character>>) -> Result<bool, Self::Error>;
    fn set_global_channels(&self, channels: Cow<Vec<(Channel, u32)>>) -> Result<bool, Self::Error>;
    fn set_unofficial_channels(&self, channels: Cow<Vec<(Channel, u32)>>) -> Result<bool, Self::Error>;
    
    fn get_channel(&self, channel: &Channel) -> Result<Option<ChannelData>, Self::Error>;
    fn get_channels(&self) -> Result<Vec<ChannelData>, Self::Error>;
    fn get_character(&self, character: &Character) -> Result<Option<CharacterData>, Self::Error>;
    fn get_characters(&self) -> Result<Vec<CharacterData>, Self::Error>;
    fn get_messages(&self, source: &MessageChannel, since: Option<Timestamp>, limit: Option<u32>) -> Result<Vec<Message>, Self::Error>;
    fn get_friend_relations(&self) -> Result<Vec<FriendRelation>, Self::Error>;
    fn get_friends(&self) -> Result<Vec<Character>, Self::Error> {
        self.get_friend_relations().map(|mut v|v.drain(..).map(|c|c.other_character).collect())
    }
    fn get_bookmarks(&self) -> Result<Vec<Character>, Self::Error>;
}

#[derive(Serialize, Debug, Default)]
pub struct PartialChannelData<'a> {
    mode: Option<ChannelMode>,
    title: Option<Cow<'a, str>>,
    description: Option<Cow<'a, str>>
}

#[derive(Serialize, Debug, Default)]
pub struct PartialUserData<'a> {
    gender: Option<Gender>,
    status: Option<Status>,
    status_message: Option<Cow<'a, str>>
}

#[derive(thiserror::Error, Debug)]
pub enum NoCacheError {} // Never construct any instances of this

/// Basic no-op cache which always emits update events
#[derive(Debug)]
pub struct NoCache;
impl Cache for NoCache {
    type Error = NoCacheError;

    fn insert_message(&self, source: MessageChannel, message: Message) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn insert_channel(&self, channel: Cow<Channel>, data: PartialChannelData, members: Cow<Vec<Character>>) -> Result<bool, Self::Error> {
        Ok(true)
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

    fn get_messages(&self, source: &MessageChannel, since: Option<Timestamp>, limit: Option<u32>) -> Result<Vec<Message>, Self::Error> {
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

    fn add_bookmark(&self, character: Cow<Character>) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn remove_bookmark(&self, character: Cow<Character>) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn set_global_channels(&self, channels: Cow<Vec<Channel>>) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn set_unofficial_channels(&self, channels: Cow<Vec<Channel>>) -> Result<bool, Self::Error> {
        Ok(true)
    } 
}