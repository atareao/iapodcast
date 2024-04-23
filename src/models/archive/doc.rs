use serde::{Serialize, Deserialize, Deserializer, de};
use chrono::{DateTime, Utc};
use tracing::debug;
use tokio::fs;
use std::{fmt, marker::PhantomData};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct Doc{
    #[serde(default = "default_number")]
    number: usize,
    identifier: String,
    #[serde(default = "default_number")]
    version: usize,
    publicdate: DateTime<Utc>,
    #[serde(deserialize_with = "string_or_seq_string")]
    subject: Vec<String>,
    description: String,
    title: String,
    downloads: u64,
}

fn default_number() -> usize {
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

impl Doc{
    pub fn get_number(&self) -> usize{
        self.number
    }
    pub fn set_number(&mut self, number: usize) {
        self.number = number;
    }
    pub fn get_version(&self) -> usize{
        self.version
    }

    pub fn set_version(&mut self, version: usize) {
        self.version = version;
    }

    pub fn get_datetime(&self) ->DateTime<Utc>{
        self.publicdate
    }

    pub fn get_identifier(&self) -> &str{
        &self.identifier
    }

    pub fn get_subject(&self) -> Vec<String>{
        self.subject.clone()
    }
    pub fn get_description(&self) -> &str{
        &self.description
    }
    pub fn get_filename(&self) -> String{
        format!("{}.md", self.identifier)
    }
    pub fn get_downloads(&self) -> u64{
        self.downloads
    }

    pub async fn exists(&self) -> bool{
        let file = format!("episodes/{}", self.get_filename());
        match fs::metadata(&file).await{
            Ok(metadata) => {
                debug!("Output file {} exists", &file);
                metadata.is_file()
            },
            Err(e) => {
                debug!("Output file {} not exists. {}", &file, e);
                false
            },
        }
    }
}

