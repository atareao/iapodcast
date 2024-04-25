use gray_matter::{Matter, engine::YAML};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use comrak::{markdown_to_html, ComrakOptions};

use super::{
    Post,
    super::utils::{
        get_slug,
        get_unix_time,
        get_excerpt,
        string_or_seq_string,
    }
};

#[derive(Debug, Serialize, Deserialize)]
struct Metadata{
    pub title: String,
    pub date: String,
    #[serde(deserialize_with = "string_or_seq_string")]
    pub subject: Vec<String>,
}

impl Metadata{
    pub fn get_slug(&self) -> String {
        get_slug(&self.title)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page{
    metadata: Metadata,
    filename: String,
    pub content: String,
}

impl Page{
    pub fn get_post(&self) -> Post{
        info!("get_post");
        let options = &ComrakOptions::default();
        let date = get_unix_time(&self.metadata.date);
        let slug = self.metadata.get_slug();
        let content = markdown_to_html(&self.content, options);
        let excerpt = markdown_to_html(get_excerpt(&self.content), options);
        Post{
            title: self.metadata.title.clone(),
            date,
            excerpt,
            slug,
            content,
            subject: self.metadata.subject.clone(),
            identifier: self.filename.clone(),
            filename: self.filename.clone(),
            size: 0,
            length: 0,
            number: 0,
            downloads: 0,
        }
    }

    pub async fn new(filename: &str) -> Result<Self, serde_json::Error>{
        info!("new: {filename}");
        let filename = format!("pages/{}", filename);
        debug!("Filename: {}", filename);
        let data = tokio::fs::read_to_string(&filename)
            .await
            .unwrap();
        let matter = Matter::<YAML>::new();
        let result = matter.parse(&data);
        let metadata: Metadata = result.data.unwrap().deserialize()?;
        let page = Page{
            metadata,
            filename: filename.to_string(),
            content: result.content,
        };
        Ok(page)
    }
}


#[cfg(test)]
mod tests {
    use tracing_subscriber::{
        EnvFilter,
        layer::SubscriberExt,
        util::SubscriberInitExt
    };
    use std::str::FromStr;
    use tracing::debug;
    use super::Page;

    #[tokio::test]
    async fn test_page(){
        tracing_subscriber::registry()
            .with(EnvFilter::from_str("debug").unwrap())
            .with(tracing_subscriber::fmt::layer())
            .init();

        let page = Page::new("about.md").await.unwrap();
        debug!("Title: {}", page.metadata.title);
        debug!("=========================");
        debug!("{:?}", page);
        debug!("=========================");
        assert!(!page.metadata.title.is_empty());
    }
}

