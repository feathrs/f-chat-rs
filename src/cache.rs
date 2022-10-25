// Cache trait for the client. 
use std::borrow::Cow;

use serde::Serialize;

use crate::{data::{Character, Channel, ChannelMode, Gender, Status}, client::MessageSource, util::timestamp::Timestamp};

pub trait Cache {
    type Error: std::error::Error;

    fn insert_message(message: MessageData) -> Result<(), Self::Error>;
    fn insert_channel(channel: Cow<Channel>, data: PartialChannelData, members: Vec<Character>);

    fn add_channel_member(channel: Cow<Channel>, member: Character) -> Result<bool, Self::Error>;
    fn remove_channel_member(channel: Cow<Channel>, member: Character) -> Result<bool, Self::Error>;

    fn update_channel(channel: Cow<Channel>, data: PartialChannelData) -> Result<bool, Self::Error>;
    fn update_character(character: Cow<Character>, data: PartialUserData) -> Result<bool, Self::Error>;
    
    fn get_channel(channel: &Channel) -> Result<Option<ChannelData>, Self::Error>;
    fn get_channels() -> Result<Vec<ChannelData>, Self::Error>;
    fn get_character(character: &Character) -> Result<Option<CharacterData>, Self::Error>;
    fn get_characters() -> Result<Vec<CharacterData>, Self::Error>;
    fn get_messages(source: MessageSource, since: Option<Timestamp>, limit: Option<u32>) -> Result<Vec<MessageData>, Self::Error>;
}

#[derive(Serialize, Debug)]
pub struct PartialChannelData {
    mode: Option<ChannelMode>,
    title: Option<String>,
    description: Option<String>
}

#[derive(Serialize, Debug)]
pub struct PartialUserData {

}

#[derive(Debug, Default, Clone, Serialize)]
pub struct ChannelData {
    channel_mode: ChannelMode,
    members: Vec<Character>,
    description: String,
    title: String,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct CharacterData {
    pub gender: Gender,
    pub status: Status,
    pub status_message: String,
}

pub struct MessageData {

};


#[derive(thiserror::Error, Debug)]
pub enum NoCacheError {} // Never construct any instances of this

#[derive(Debug)]
pub struct NoCache;
impl Cache for NoCache {
    // Basic no-op cache which always emits update events
    type Error = NoCacheError;

    fn insert_message(message: MessageData) -> Result<(), Self::Error> {
        Ok(())
    }

    fn insert_channel(channel: Cow<Channel>, data: PartialChannelData, members: Vec<Character>) {
        todo!()
    }

    fn update_channel(channel: Cow<Channel>, data: PartialChannelData) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn update_character(character: Cow<Character>, data: PartialUserData) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn get_channel(channel: &Channel) -> Result<Option<ChannelData>, Self::Error> {
        Ok(None)
    }

    fn get_channels() -> Result<Vec<ChannelData>, Self::Error> {
        Ok(Vec::new())
    }

    fn get_character(character: &Character) -> Result<Option<CharacterData>, Self::Error> {
        Ok(None)
    }

    fn get_characters() -> Result<Vec<CharacterData>, Self::Error> {
        Ok(Vec::new())
    }

    
}