use serde::{Deserialize, Serialize};
use rss::{
    ChannelBuilder,
    ImageBuilder,
    CategoryBuilder,
    Item,
    extension::itunes::ITunesChannelExtensionBuilder
};
use super::{
    config::Podcast,
    Error,
};

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
    pub fn rss(&self, episodes: Vec<Item>) -> Result<String, Error>{
        let image = ImageBuilder::default()
            .url(&self.podcast.image_url)
            .build();
        let category = CategoryBuilder::default()
            .name(&self.podcast.category)
            .build();
        let itunes = ITunesChannelExtensionBuilder::default()
            .author(Some(self.podcast.author.clone()))
            .build();
        let mut channel = ChannelBuilder::default()
            .title(&self.podcast.title)
            .link(&self.podcast.link)
            .image(Some(image))
            .category(category)
            .rating(Some(self.rating.clone()))
            .description(self.description.clone())
            .build();
        channel.set_itunes_ext(itunes);
        channel.set_items(episodes);
        channel.pretty_write_to(std::io::sink(), b' ', 4)?;
        Ok(channel.to_string())
    }
}
