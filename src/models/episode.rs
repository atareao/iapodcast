use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Deserializer, de};
use tracing::{debug, info, error};
use gray_matter::{Matter, engine::YAML};
use comrak::{markdown_to_html, ComrakOptions};
use std::{fmt, marker::PhantomData};

use super::{
    archive::{
        Doc,
        IAMetadata,
        Mp3Metadata,
    },
    config::{Post, Podcast},
    utils::{
        get_slug,
        get_excerpt
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata{
    // from doc
    #[serde(default = "default_number")]
    pub number: usize,
    pub identifier: String,
    pub title: String,
    #[serde(deserialize_with = "string_or_seq_string")]
    pub subject: Vec<String>,
    //pub description: String,
    pub downloads: u64,
    // from mp3 metadata
    pub filename: String,
    #[serde(default = "get_default_datetime")]
    pub datetime: Option<DateTime<Utc>>,
    #[serde(default = "get_default_version")]
    pub version: usize,
    pub size: u64,
    pub length: u64,
    pub excerpt: String,
    //pub comment: String,
    // more
    pub slug: String,
}

fn get_default_datetime() -> Option<DateTime<Utc>>{
    None
}

fn get_default_version() -> usize{
    0
}


fn string_or_seq_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where D: Deserializer<'de>
{
    struct StringOrVec(PhantomData<Vec<String>>);

    impl<'de> de::Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where E: de::Error
        {
            Ok(vec![value.to_owned()])
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
            where S: de::SeqAccess<'de>
        {
            Deserialize::deserialize(de::value::SeqAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(StringOrVec(PhantomData))
}


fn default_number() -> usize {
    0
}

impl Metadata{
    pub fn get_filename(&self) -> String {
        format!("episodes/{}.md", self.identifier)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Episode{
    metadata: Metadata,
    pub content: String,
}

impl Episode{
    pub fn get_post(&self) -> Post{
        let content = markdown_to_html(&self.content, &ComrakOptions::default());
        Post{
            slug: self.metadata.slug.clone(),
            excerpt: self.metadata.excerpt.clone(),
            title: self.metadata.title.clone(),
            content,
            subject: self.metadata.subject.clone(),
            date: self.metadata.datetime.unwrap(),
            version: self.metadata.version,
            identifier: self.metadata.identifier.clone(),
            filename: self.metadata.filename.clone(),
            length: self.metadata.length,
            size: self.metadata.size,
            number: self.metadata.number,
            downloads: self.metadata.downloads,
        }
    }

    pub fn set_version(&mut self, version: usize){
        self.metadata.version = version
    }

    pub fn get_version(&self) -> usize{
        self.metadata.version
    }

    pub fn set_datetime(&mut self, datetime: DateTime<Utc>){
        self.metadata.datetime = Some(datetime);
    }

    pub async fn new(filename: &str) -> Result<Self, serde_json::Error>{
        let mut save = false;
        let filename = format!("episodes/{}", filename);
        debug!("Filename: {}", filename);
        let data = tokio::fs::read_to_string(&filename)
            .await
            .unwrap();
        let matter = Matter::<YAML>::new();
        let result = matter.parse(&data);
        let mut metadata: Metadata = result.data.unwrap().deserialize()?;
        debug!("Metadata: {:?}", &metadata);
        if metadata.slug.is_empty(){
            debug!("Is empty");
            metadata.slug = get_slug(&metadata.title);
            save = true;
        }
        if metadata.excerpt.is_empty(){
            metadata.excerpt = match result.excerpt {
                Some(excerpt) => {
                    save = true;
                    excerpt
                },
                None => get_excerpt(&result.content).to_string(),
            };
        }
        debug!("Metadata: {:?}", &metadata);
        let episode = Self{
            metadata,
            content: result.content,
        };
        if save{
            match episode.save().await{
                Ok(_) => {
                    info!("Saved article {}", episode.get_filename());
                    if filename != episode.get_filename(){
                        match tokio::fs::remove_file(&filename).await{
                            Ok(_) => info!("Removed {}", &filename),
                            Err(e) => error!("Cant remove {}. {}", &filename, e),
                        }
                    }
                },
                Err(_) => error!("Cant save article {}", episode.get_filename()),
            }
        }
        Ok(episode)
    }

    #[allow(dead_code)]
    pub fn get_title(&self) -> &str{
        &self.metadata.title
    }

    pub fn get_filename(&self) -> String{
        self.metadata.get_filename()
    }

    pub fn get_slug(&self) -> String{
        self.metadata.slug.to_string()
    }

    pub fn get_downloads(&self) -> u64{
        self.metadata.downloads
    }

    pub fn set_downloads(&mut self, downloads: u64){
        self.metadata.downloads = downloads;
    }

    pub async fn save(&self)-> tokio::io::Result<()>{
        info!("save {}", &self.get_filename());
        let mut content = String::new();
        debug!("Metadata: {}", &serde_yaml::to_string(&self.metadata).unwrap());
        content.push_str("---\n");
        content.push_str(&serde_yaml::to_string(&self.metadata).unwrap());
        content.push_str("---\n");
        content.push_str(&self.content);
        debug!("Content: {}", content);
        tokio::fs::write(self.get_filename(), content).await
    }

    pub fn combine(doc: &Doc, iametadata: &IAMetadata, mp3: &Mp3Metadata) -> Episode{
        let title = if mp3.title.is_empty(){
            doc.get_identifier()
        }else{
            &mp3.title
        };
        let comment = if mp3.comment.is_empty(){
            debug!("Description: {}", &iametadata.description);
            get_excerpt(&iametadata.description)
        }else{
            &mp3.comment
        };
        debug!("Comment: {}", &comment);
        let metadata = Metadata{
            number: doc.get_number(),
            identifier: doc.get_identifier().to_string(),
            subject: doc.get_subject(),
            downloads: doc.get_downloads(),
            datetime: Some(doc.get_datetime()),
            version: doc.get_version(),
            title: title.to_string(),
            excerpt: comment.to_owned(),
            filename: mp3.filename.to_string(),
            size: mp3.size,
            length: mp3.length,
            slug: get_slug(title),
        };
        Self{
            metadata,
            content: iametadata.description.to_owned()
        }
    }
}
