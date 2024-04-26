use serde::{Serialize, Deserialize, Deserializer};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Podcast{
    pub base_url: String,
    pub feed_url: String,
    pub url: String,
    pub author: String,
    pub email: String,
    pub image_url: String,
    pub category: String,
    #[serde(deserialize_with = "empty_as_none")]
    pub subcategory: Option<String>,
    pub explicit: bool,
    pub title: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub license: String,
}

fn empty_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    }else{
        Ok(Some(s.to_string()))
    }
}
