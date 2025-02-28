use serde::{Deserialize, Serialize};
use crate::story::comment::Comment;
use crate::story::item::StoryItem;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StoryData {
    #[serde(flatten)]
    pub item: StoryItem,
    #[serde(default)]
    pub comments: Vec<Comment>,
}