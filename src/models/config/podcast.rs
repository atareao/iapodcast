use rss::{
    extension::itunes::{
        ITunesChannelExtensionBuilder,
        ITunesCategoryBuilder,
        ITunesCategory,
        ITunesOwnerBuilder,
    },
    Channel,
    ImageBuilder,
    CategoryBuilder,
    ChannelBuilder
};
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Podcast{
    pub author: String,
    pub email: String,
    pub link: String,
    pub image_url: String,
    pub categories: Vec<String>,
    pub rating: String,
    pub explicit: bool,
    pub title: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub license: String,
}

impl Podcast {
    pub fn get_channel(&self) -> Channel {
        let itunes_categories: Vec<ITunesCategory> = self.categories.iter()
            .map(|c| ITunesCategoryBuilder::default()
                .text(c)
                .build()
            )
            .collect();
        let keywords = self.keywords.join(",");
        let owner = ITunesOwnerBuilder::default()
            .name(Some(self.author))
            .email(Some(self.email))
            .build();
        let itunes = ITunesChannelExtensionBuilder::default()
            .author(Some(self.author))
            .categories(itunes_categories)
            .image(Some(self.image_url))
            .explicit(Some(self.explicit.to_string()))
            .owner(Some(owner))
            .subtitle(Some(self.title))
            .summary(Some(self.description))
            .keywords(Some(keywords))
            .build();
        let image = ImageBuilder::default()
            .url(&self.image_url)
            .build();
        let categories: Vec<rss::Category> = self.categories.iter()
            .map(|c| CategoryBuilder::default()
                .name(c)
                .build()
            )
            .collect();
        let now = chrono::Local::now().to_rfc2822();
        let copyright = format!("By {author} under {license}",
            author=self.author, license=self.license);
        ChannelBuilder::default()
            .title(self.title)
            .link(self.link)
            .description(self.description)
            .categories(categories)
            .copyright(Some(copyright))
            .managing_editor(Some(self.email))
            .webmaster(Some(self.email))
            .pub_date(Some(now))
            .last_build_date(Some(now))
            .generator(Some("IAPodcast".to_string()))
            .image(Some(image))
            .itunes_ext(Some(itunes))
            .build()
    }
}
