use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Site{
    pub author: String,
    pub title: String,
    pub description: String,
    pub podcast_feed: String,
    pub baseurl: String,
    pub url: String,
    pub avatar: String,
    pub category: String,
    pub subcategory: String,
    pub explicit: bool,
}
