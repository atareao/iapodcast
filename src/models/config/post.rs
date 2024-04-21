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

use crate::models::utils::from_sec;

use super::Podcast;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Post{
    pub podcast: Podcast,
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

impl Into<Item> for Post{
    fn into(self) -> Item{
        let link = format!("{url}/{slug}", url=self.podcast.url,
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
            .url(url)
            .length(self.length.to_string())
            .mime_type(mime_type)
            .build();
        let guid = GuidBuilder::default()
            .value(self.identifier)
            .build();
        let source = SourceBuilder::default()
            .url(url)
            .build();

        let keywords: String = self.subject.join(",");
        let itunes_ext = ITunesItemExtensionBuilder::default()
            .author(Some(self.podcast.author))
            .duration(Some(from_sec(self.length)))
            .explicit(Some(self.podcast.explicit.to_string()))
            .subtitle(Some(self.title))
            .summary(Some(self.content))
            .keywords(Some(keywords))
            .episode(Some(self.number.to_string()))
            .build();
        ItemBuilder::default()
            .title(Some(self.title))
            .link(Some(link))
            .description(Some(self.content))
            .author(Some(self.podcast.author))
            .categories(categories)
            .enclosure(Some(enclosure))
            .guid(Some(guid))
            .pub_date(Some(self.date.to_rfc2822()))
            .source(Some(source))
            .content(Some(self.content))
            .itunes_ext(Some(itunes_ext))
            .build()
    }
}

