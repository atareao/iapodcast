use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, error};
use gray_matter::{Matter, engine::YAML};
use comrak::{markdown_to_html, ComrakOptions};

use super::utils::string_or_seq_string;

use super::{
    archive::Doc,
    config::Post,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
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

fn default_number() -> usize {
    0
}

impl Metadata{
    pub fn get_filename(&self) -> String {
        format!("episodes/{}.md", &self.identifier)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Episode{
    metadata: Metadata,
    pub content: String,
}

impl Episode{
    pub fn get_filename(&self) -> String {
        self.metadata.get_filename()
    }

    pub fn get_identifier(&self) -> &str {
        self.metadata.identifier.as_str()
    }

    pub fn get_post(&self) -> Post{
        let options = &ComrakOptions::default();
        let content = markdown_to_html(&self.content, options);
        let excerpt = markdown_to_html(&self.metadata.excerpt, options);
        Post{
            slug: self.metadata.slug.clone(),
            excerpt,
            title: self.metadata.title.clone(),
            content,
            subject: self.metadata.subject.clone(),
            date: self.metadata.datetime.unwrap(),
            identifier: self.metadata.identifier.clone(),
            filename: self.metadata.filename.clone(),
            length: self.metadata.length,
            size: self.metadata.size,
            number: self.metadata.number,
            downloads: self.metadata.downloads,
        }
    }

    pub async fn new(filename: &str) -> Result<Self, serde_json::Error>{
        let filename = format!("episodes/{}", filename);
        debug!("Filename: {}", filename);
        let data = tokio::fs::read_to_string(&filename)
            .await
            .unwrap();
        let matter = Matter::<YAML>::new();
        let result = matter.parse(&data);
        let metadata: Metadata = result.data.unwrap().deserialize()?;
        debug!("Metadata: {:?}", &metadata);
        let episode = Self{
            metadata,
            content: result.content,
        };
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
        Ok(episode)
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
}

impl From<Doc> for Episode{
    fn from(doc: Doc) -> Self{
        let metadata = Metadata{
            number: doc.get_number(),
            identifier: doc.get_identifier().to_string(),
            subject: doc.get_subject(),
            downloads: doc.get_downloads(),
            datetime: Some(doc.get_datetime()),
            title: doc.get_title().to_string(),
            excerpt: doc.get_exceprt(),
            filename: doc.get_audio_filename().to_string(),
            size: doc.get_size(),
            length: doc.get_length(),
            slug: doc.get_slug(),
        };
        Self{
            metadata,
            content: doc.get_description().to_string()
        }
    }
}
