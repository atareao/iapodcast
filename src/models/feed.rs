use serde::{Deserialize, Serialize};
use super::{
    config::{
        Podcast,
        Post
    },
    Error,
};
use rss::Item;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Feed{
    podcast: Podcast,
}

impl Feed {
    pub fn new(podcast: Podcast) -> Self{
        Self{
            podcast,
        }
    }
    pub fn rss(&self, posts: Vec<Post>) -> Result<String, Error>{
        let mut channel = self.podcast.get_channel();
        let items: Vec<Item> = posts.iter()
            .map(|post| post.get_item(&self.podcast))
            .collect();
        channel.set_items(items);
        channel.pretty_write_to(std::io::sink(), b' ', 4)?;
        Ok(channel.to_string())
    }
}
