use serde::{Serialize, Deserialize, Deserializer, de};
use regex::Regex;
use chrono::{DateTime, Utc};
use tracing::{info, error, debug};
use tokio::fs;
use std::{fmt, marker::PhantomData};

use super::{
    BASE_URL,
    super::{
        error::Error,
        utils::{
            get_slug,
            get_excerpt
        },
    },
};

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
    // audio
    #[serde(default = "default_empty")]
    filename: String,
    #[serde(default = "default_u64")]
    mtime: u64,
    #[serde(default = "default_u64")]
    size: u64,
    #[serde(default = "default_u64")]
    length: u64,
}

fn default_empty() -> String {
    "".to_string()
}

fn default_number() -> usize {
    0
}

fn default_u64() -> u64 {
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
    pub fn complete(&mut self) -> Result<(), Error>{
        let identifier = &self.identifier;
        let url = format!("{}/download/{identifier}/{identifier}_files.xml",
            BASE_URL, identifier=identifier);
        info!("url: {}", url);
        let response = ureq::get(&url).call()?;
        if response.status() != 200{
            let status_code = response.status();
            let message = response.into_string().unwrap();
            let error_message = format!("HTTP Error: {status_code}. Error: {message}");
            error!("{error_message}");
            return Err(Error::new(&error_message));
        }
        let content = response.into_string()?;
        let pattern_init = Regex::new(r#"^\s+<file name=".*\.(mp3|m4a)" source="original">"#).unwrap();
        let pattern_end = Regex::new(r#"^\s+</file>"#).unwrap();
        let mut audio = false;
        let mut mp3_metadata: Vec<String> = Vec::new();
        for line in content.lines(){
            if !audio && pattern_init.is_match(line){
                audio = true;
            }
            if audio{
                mp3_metadata.push(line.to_string());
            }
            if audio && pattern_end.is_match(line){
                break;
            }
        }
        let text = mp3_metadata.concat();
        if !text.is_empty(){
            debug!("Text: {}", &text);
            let mtime = Self::get_value("mtime", &text).parse().unwrap();
            let size = Self::get_value("size", &text).parse().unwrap();
            let length = Self::get_value("length", &text);
            let length = match length.find('.'){
                Some(pos) => length.get(0..pos).unwrap().parse().unwrap(),
                None => length.parse().unwrap(),
            };
            let pattern = r#"<file name="([^"]*)" source="original">"#;
            let re = Regex::new(pattern).unwrap();
            let caps = re.captures(&text).unwrap();
            let filename = caps.get(1).unwrap().as_str().to_string();
            self.filename = filename;
            self.mtime = mtime;
            self.size = size;
            self.length = length;
        }
        info!("complete doc");
        Ok(())
    }
    pub fn get_number(&self) -> usize{
        self.number
    }
    pub fn set_number(&mut self, number: usize) {
        self.number = number;
    }
    pub fn get_version(&self) -> usize{
        self.version
    }

    pub fn get_title(&self) -> &str{
        self.title.as_str()
    }

    pub fn set_version(&mut self, version: usize) {
        self.version = version;
    }

    pub fn get_datetime(&self) ->DateTime<Utc>{
        self.publicdate
    }

    pub fn get_identifier(&self) -> &str{
        self.identifier.as_str()
    }

    pub fn get_subject(&self) -> Vec<String>{
        self.subject.clone()
    }
    pub fn get_description(&self) -> &str{
        self.description.as_str()
    }

    pub fn get_exceprt(&self) -> String{
        get_excerpt(&self.description).to_string()
    }

    pub fn get_slug(&self) -> String {
        get_slug(&self.title).to_string()
    }

    pub fn get_post_filename(&self) -> String{
        format!("{}.md", self.identifier)
    }
    pub fn get_downloads(&self) -> u64{
        self.downloads
    }

    pub fn get_audio_filename(&self) -> &str{
        self.filename.as_str()
    }

    pub fn get_mtime(&self) -> u64 {
        self.mtime
    }

    pub fn get_size(&self) -> u64 {
        self.size
    }

    pub fn get_length(&self) -> u64 {
        self.length
    }

    pub async fn exists(&self) -> bool{
        let file = format!("episodes/{}", self.get_post_filename());
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

    fn get(tag: &str, xml: &str) -> Vec<String>{
        let mut result = Vec::new();
        let pattern = format!("<{tag}>([^<]*)</{tag}>", tag=tag);
        let re = Regex::new(&pattern).unwrap();

        for cap in re.captures_iter(xml){
            result.push(cap.get(1).unwrap().as_str().to_string());
        }
        result
    }
    fn get_value(tag: &str, xml: &str) -> String{
        let value = Self::get(tag, xml);
        if !value.is_empty(){
            match value.first() {
                Some(value) => value.to_string(),
                None => "".to_string()
            }
        }else{
            "".to_string()
        }

    }
}

