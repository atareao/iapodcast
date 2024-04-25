use rss::{
    GuidBuilder,
    SourceBuilder
};
use serde::{Serialize, Deserialize};
use rss::{
    Item,
    extension::itunes::ITunesItemExtensionBuilder,
    ItemBuilder,
    CategoryBuilder,
    Category, EnclosureBuilder};
use chrono::{DateTime, Utc};
use std::path::Path;
use std::ffi::OsStr;
use super::Podcast;

use crate::models::utils::from_sec;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Post{
    pub slug: String,
    pub excerpt: String,
    pub title: String,
    pub content: String,
    pub subject: Vec<String>,
    pub date: DateTime<Utc>,
    pub identifier: String,
    pub filename: String,
    pub size: u64,
    pub length: u64,
    pub number: usize,
    pub downloads: u64,
}

impl Post{
    pub fn get_item(&self, podcast: &Podcast) -> Item{
        let link = format!("{url}/{slug}", url=podcast.url,
            slug=self.slug);
        let categories: Vec<Category> = self.subject.iter()
            .map(|c| CategoryBuilder::default()
                .name(c)
                .build()
            )
            .collect();
        let url = format!("https://archive.org/{identifier}/{filename}", identifier=self.identifier, filename=self.filename);
        let extension = Path::new(&self.filename)
            .extension()
            .and_then(OsStr::to_str)
            .unwrap();
        let mime_type = format!("audio/{extension}");
        let enclosure = EnclosureBuilder::default()
            .url(url.clone())
            .length(self.length.to_string())
            .mime_type(mime_type)
            .build();
        let guid = GuidBuilder::default()
            .value(self.identifier.to_string())
            .build();
        let source = SourceBuilder::default()
            .url(url)
            .build();

        let keywords: String = self.subject.join(",");
        let itunes_ext = ITunesItemExtensionBuilder::default()
            .author(Some(podcast.author.to_string()))
            .duration(Some(from_sec(self.length)))
            .explicit(Some(podcast.explicit.to_string()))
            .subtitle(Some(self.title.to_string()))
            .summary(Some(self.content.to_string()))
            .keywords(Some(keywords))
            .episode(Some(self.number.to_string()))
            .build();
        ItemBuilder::default()
            .title(Some(self.title.to_string()))
            .link(Some(link))
            .description(Some(self.content.to_string()))
            .author(Some(podcast.author.to_string()))
            .categories(categories)
            .enclosure(Some(enclosure))
            .guid(Some(guid))
            .pub_date(Some(self.date.to_rfc2822()))
            .source(Some(source))
            .content(Some(self.content.to_string()))
            .itunes_ext(Some(itunes_ext))
            .build()
    }
}

