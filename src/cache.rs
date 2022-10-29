// Cache trait for the client. 
use std::borrow::Cow;

use serde::Serialize;

use crate::{data::{Character, Channel, ChannelMode, Gender, Status, FriendRelation, MessageChannel, Message, ChannelData, CharacterData}, util::timestamp::Timestamp};

pub trait Cache: std::marker::Sync + Sized + std::marker::Send {
    type Error: std::error::Error;

    fn insert_message(&self, source: MessageChannel, message: Message) -> Result<bool, Self::Error>;
    fn insert_channel(&self, channel: Cow<Channel>, data: PartialChannelData, members: Cow<[Character]>) -> Result<bool, Self::Error>;
    fn insert_ad(&self, channel: Cow<Channel>, character: Cow<Character>, ad: Cow<str>) -> Result<bool, Self::Error>;

    fn add_channel_member(&self, channel: Cow<Channel>, member: Character) -> Result<bool, Self::Error>;
    fn remove_channel_member(&self, channel: Cow<Channel>, member: Character) -> Result<bool, Self::Error>;

    fn add_bookmark(&self, character: Cow<Character>) -> Result<bool, Self::Error>;
    fn remove_bookmark(&self, character: Cow<Character>) -> Result<bool, Self::Error>;

    fn add_global_op(&self, character: Cow<Character>) -> Result<bool, Self::Error>;
    fn remove_global_op(&self, character: Cow<Character>) -> Result<bool, Self::Error>;

    fn add_channel_op(&self, channel: Cow<Channel>, character: Cow<Character>) -> Result<bool, Self::Error>;
    fn remove_channel_op(&self, channel: Cow<Channel>, character: Cow<Character>) -> Result<bool, Self::Error>;

    fn update_channel(&self, channel: Cow<Channel>, data: PartialChannelData) -> Result<bool, Self::Error>;
    fn update_character(&self, character: Cow<Character>, data: PartialUserData) -> Result<bool, Self::Error>;
    
    fn set_friends(&self, friends: Cow<[FriendRelation]>) -> Result<bool, Self::Error>;
    fn set_bookmarks(&self, bookmarks: Cow<[Character]>) -> Result<bool, Self::Error>;
    fn set_channel_members(&self, channel: Cow<Channel>, members: Cow<[Character]>) -> Result<bool, Self::Error>;
    fn set_global_channels(&self, channels: Cow<[(Channel, u32)]>) -> Result<bool, Self::Error>;
    fn set_unofficial_channels(&self, channels: Cow<[(Channel, u32)]>) -> Result<bool, Self::Error>;
    fn set_global_ops(&self, ops: Cow<[Character]>) -> Result<bool, Self::Error>;
    fn set_channel_ops(&self, channel: Cow<Channel>, ops: Cow<[Character]>) -> Result<bool, Self::Error>;
    
    fn get_channel(&self, channel: &Channel) -> Result<Option<ChannelData>, Self::Error>;
    fn get_channels(&self) -> Result<Cow<[ChannelData]>, Self::Error>;
    fn get_character(&self, character: &Character) -> Result<Option<CharacterData>, Self::Error>;
    fn get_characters(&self) -> Result<Cow<[CharacterData]>, Self::Error>;
    fn get_messages(&self, source: &MessageChannel, since: Option<Timestamp>, limit: Option<u32>) -> Result<Cow<[Message]>, Self::Error>;
    fn get_friend_relations(&self) -> Result<Cow<[FriendRelation]>, Self::Error>;
    fn get_friends(&self) -> Result<Cow<[Character]>, Self::Error> {
        self.get_friend_relations().map(|v|v.iter().map(|c|c.other_character).collect())
    }
    fn get_bookmarks(&self) -> Result<Cow<[Character]>, Self::Error>;
}

#[derive(Serialize, Debug, Default)]
pub struct PartialChannelData<'a> {
    pub mode: Option<ChannelMode>,
    pub title: Option<Cow<'a, str>>,
    pub description: Option<Cow<'a, str>>
}

#[derive(Serialize, Debug, Default)]
pub struct PartialUserData<'a> {
    pub gender: Option<Gender>,
    pub status: Option<Status>,
    pub status_message: Option<Cow<'a, str>>
}

#[derive(thiserror::Error, Debug)]
pub enum NoCacheError {} // Never construct any instances of this

/// Basic no-op cache which always emits update events
#[derive(Debug)]
pub struct NoCache;
impl Cache for NoCache {
    type Error = NoCacheError;

    fn insert_message(&self, _source: MessageChannel, _message: Message) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn insert_channel(&self, _channel: Cow<Channel>, _data: PartialChannelData, _members: Cow<[Character]>) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn update_channel(&self, _channel: Cow<Channel>, _data: PartialChannelData) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn update_character(&self, _character: Cow<Character>, _data: PartialUserData) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn get_channel(&self, _channel: &Channel) -> Result<Option<ChannelData>, Self::Error> {
        Ok(None)
    }

    fn get_channels(&self) -> Result<Cow<[ChannelData]>, Self::Error> {
        Ok(Vec::new().into())
    }

    fn get_character(&self, _character: &Character) -> Result<Option<CharacterData>, Self::Error> {
        Ok(None)
    }

    fn get_characters(&self) -> Result<Cow<[CharacterData]>, Self::Error> {
        Ok(Vec::new().into())
    }

    fn add_channel_member(&self, _channel: Cow<Channel>, _member: Character) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn remove_channel_member(&self, _channel: Cow<Channel>, _member: Character) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn get_messages(&self, _source: &MessageChannel, _since: Option<Timestamp>, _limit: Option<u32>) -> Result<Cow<[Message]>, Self::Error> {
        Ok(Vec::new().into())
    }

    fn set_friends(&self, _friends: Cow<[FriendRelation]>) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn set_bookmarks(&self, _bookmarks: Cow<[Character]>) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn set_channel_members(&self, _channel: Cow<Channel>, _members: Cow<[Character]>) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn get_friend_relations(&self) -> Result<Cow<[FriendRelation]>, Self::Error> {
        Ok(Vec::new().into())
    }

    fn get_bookmarks(&self) -> Result<Cow<[Character]>, Self::Error> {
        Ok(Vec::new().into())
    }

    fn add_bookmark(&self, _character: Cow<Character>) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn remove_bookmark(&self, _character: Cow<Character>) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn set_global_channels(&self, _channels: Cow<[(Channel, u32)]>) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn set_unofficial_channels(&self, _channels: Cow<[(Channel, u32)]>) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn add_global_op(&self, _character: Cow<Character>) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn remove_global_op(&self, _character: Cow<Character>) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn add_channel_op(&self, _channel: Cow<Channel>, _character: Cow<Character>) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn remove_channel_op(&self, _channel: Cow<Channel>, _character: Cow<Character>) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn set_global_ops(&self, _ops: Cow<[Character]>) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn set_channel_ops(&self, _channel: Cow<Channel>, _ops: Cow<[Character]>) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn insert_ad(&self, _channel: Cow<Channel>, _character: Cow<Character>, _ad: Cow<str>) -> Result<bool, Self::Error> {
        Ok(true)
    } 
}