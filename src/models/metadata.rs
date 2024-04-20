use regex::Regex;
use html_escape::decode_html_entities;

#[derive(Debug)]
pub struct AOMetadata{
    pub identifier: String,
    pub description: String,
}

impl AOMetadata {
    pub fn new(content: &str) -> Self{
        let identifier = Self::get("identifier", content).first().unwrap().to_string();
        let description = html2md::parse_html(
            decode_html_entities(
                &Self::get("description", content)
                .first()
                .unwrap()
                .to_owned()).trim());
        Self{
            identifier,
            description,
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
}
