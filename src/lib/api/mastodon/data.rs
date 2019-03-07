//! Mastodon API data types

use serde_derive::{Serialize, Deserialize};

use crate::data::PostFlags;

///Generic payload for response that contains entity's information.
///
///Contains only ID and therefore can work as response's payload to most requests.
#[derive(Deserialize, Debug)]
pub struct EntityId {
    ///Identifier
    pub id: String
}

///Posts new message on timeline
#[derive(Serialize, Debug)]
pub struct NewStatus<'a> {
    status: &'a str,
    ///List of `EntityId`'s id to attach
    pub media_ids: &'a [String],
    sensitive: bool,
}

impl<'a> NewStatus<'a> {
    ///Creates new instance
    pub fn new(status: &'a str, media_ids: &'a [String], flags: &PostFlags) -> Self {
        Self {
            status,
            media_ids,
            sensitive: flags.nsfw,
        }
    }
}
