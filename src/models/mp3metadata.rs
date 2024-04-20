use tracing::debug;
use regex::Regex;

#[derive(Debug)]
pub struct Mp3Metadata{
    pub title: String,
    pub filename: String,
    pub mtime: u64,
    pub size: u64,
    pub length: u64,
    pub comment: String,
}

impl Mp3Metadata {
    pub fn new(content: &str) -> Option<Mp3Metadata>{
        let pattern_init = Regex::new(r#"^\s+<file name=".*\.mp3" source="original">"#).unwrap();
        let pattern_end = Regex::new(r#"^\s+</file>"#).unwrap();
        let mut mp3 = false;
        let mut mp3_metadata: Vec<String> = Vec::new();
        for line in content.lines(){
            if !mp3 && pattern_init.is_match(line){
                mp3 = true;
            }
            if mp3{
                mp3_metadata.push(line.to_string());
            }
            if mp3 && pattern_end.is_match(line){
                break;
            }
        }
        let text = mp3_metadata.concat();
        if text.is_empty(){
            return None;
        }
        debug!("Text: {}", &text);
        let title = Self::get_value("title", &text);
        let mtime = Self::get_value("mtime", &text).parse().unwrap();
        let size = Self::get_value("size", &text).parse().unwrap();
        let length = Self::get_value("length", &text);
        let length = match length.find('.'){
            Some(pos) => length.get(0..pos).unwrap().parse().unwrap(),
            None => length.parse().unwrap(),
        };
        let comment = Self::get_value("comment", &text);
        let pattern = r#"<file name="([^"]*)" source="original">"#;
        let re = Regex::new(pattern).unwrap();
        let caps = re.captures(&text).unwrap();
        let filename = caps.get(1).unwrap().as_str().to_string();
        Some(Mp3Metadata{
            title,
            filename,
            mtime,
            size,
            length,
            comment,
        })
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
