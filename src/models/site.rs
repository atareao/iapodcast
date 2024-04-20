use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Post{
    pub slug: String,
    pub excerpt: String,
    pub title: String,
    pub content: String,
    pub subject: Vec<String>,
    pub date: DateTime<Utc>,
    pub version: usize,
    pub identifier: String,
    pub filename: String,
    pub size: u64,
    pub length: u64,
    pub number: usize,
    pub downloads: u64,
}

