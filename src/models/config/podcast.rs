use rss::{
    extension::itunes::{
        ITunesChannelExtensionBuilder,
        ITunesCategoryBuilder,
        ITunesOwnerBuilder,
    },
    Channel,
    ImageBuilder,
    CategoryBuilder,
    ChannelBuilder
};
use rss::Item;
use serde::{Serialize, Deserialize};
use super::super::{
    error::Error,
    config::Post,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Podcast{
    pub base_url: String,
    pub feed_url: String,
    pub url: String,
    pub author: String,
    pub email: String,
    pub link: String,
    pub image_url: String,
    pub category: String,
    pub subcategory: Option<String>,
    pub explicit: bool,
    pub title: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub license: String,
}

impl Podcast {
    pub fn get_channel(&self) -> Channel {
        let mut itunes_category = ITunesCategoryBuilder::default()
            .text(&self.category)
            .build();
        if let Some(subcategory) = &self.subcategory{
            let itunes_subcategory = ITunesCategoryBuilder::default()
            .text(subcategory)
            .build();
            itunes_category.set_subcategory(Box::new(itunes_subcategory));
        }
        let itunes_categories = [itunes_category];
        let keywords = self.keywords.join(",");
        let owner = ITunesOwnerBuilder::default()
            .name(Some(self.author.to_string()))
            .email(Some(self.email.to_string()))
            .build();
        let itunes = ITunesChannelExtensionBuilder::default()
            .author(Some(self.author.to_string()))
            .categories(itunes_categories)
            .image(Some(self.image_url.to_string()))
            .explicit(Some(self.explicit.to_string()))
            .owner(Some(owner))
            .subtitle(Some(self.title.to_string()))
            .summary(Some(self.description.to_string()))
            .keywords(Some(keywords))
            .build();
        let image = ImageBuilder::default()
            .url(&self.image_url)
            .build();
        let categories: Vec<rss::Category> = vec![CategoryBuilder::default()
            .name(&self.category)
            .build()
        ];
        let now = chrono::Local::now().to_rfc2822();
        let copyright = format!("By {author} under {license}",
            author=self.author, license=self.license);
        ChannelBuilder::default()
            .title(self.title.to_string())
            .link(self.link.to_string())
            .description(self.description.to_string())
            .categories(categories)
            .copyright(Some(copyright))
            .managing_editor(Some(self.email.to_string()))
            .webmaster(Some(self.email.to_string()))
            .pub_date(Some(now.clone()))
            .last_build_date(Some(now))
            .generator(Some("IAPodcast".to_string()))
            .image(Some(image))
            .itunes_ext(Some(itunes))
            .build()
    }
    pub fn get_feed(&self, posts: &[Post]) -> Result<String, Error> {
        let mut channel = self.get_channel();
        let items: Vec<Item> = posts.iter()
            .map(|post| post.get_item(&self))
            .collect();
        channel.set_items(items);
        channel.pretty_write_to(std::io::sink(), b' ', 4)?;
        Ok(channel.to_string())

    }
}
