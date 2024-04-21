use tracing::{info, debug, error};
use serde::{Serialize, Deserialize, Deserializer};
use super::{
    IAMetadata,
    Mp3Metadata,
    Doc,
};

const BASE_URL: &str = "https://archive.org";
const PAGESIZE: usize = 200;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IAClient{
    pub uploader: String,
    pub podcast: String,
}

fn deserialize_on_empty<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where D: Deserializer<'de>{
        let o: Option<String> = Option::deserialize(deserializer)?;
        Ok(o.filter(|s| !s.is_empty()))
}

impl IAClient{
    pub fn new(uploader: &str, podcast: &str) -> Self{
        Self{
            uploader: uploader.to_string(),
            podcast: podcast.to_string(),
        }
    }

    pub fn get_all_docs(&self) -> Vec<Doc>{
        let since = "1970-01-01";
        let page = 1;
        self.get_docs(since, page)
    }

    fn get_docs(&self, since: &str, page: usize) -> Vec<Doc>{
        let mut items = Vec::new();
        let fields: String = ["description", "downloads", "identifier",
            "item_size", "name", "publicdate",
            "publisher", "subject", "title"]
            .into_iter()
            .map(|field| format!("fl[]={}", field))
            .collect::<Vec<String>>()
        .join("&");
        
        let sort = "publicdate asc";
        let output = "json";
        let q: String = [
            format!("uploader:({uploader})", uploader=self.uploader),
            format!("publicdate:[{since} TO 9999-12-31]"),
            format!("podcast:({podcast})", podcast=self.podcast),
            "mediatype:(audio)".to_string(),
        ].join(" AND ");

        let url = format!("{base}/advancedsearch.php?q=uploader:({uploader}) \
            AND date:[{since} TO 9999-12-31] \
            AND mediatype:(audio) \
            AND podcast:({podcast})
            AND format:(MP3) \
            OR format:(MPEG-4)
            &{fields}\
            &sort[]={sort}\
            &output={output}\
            &rows={rows}\
            &page={page}",
            base=BASE_URL, uploader=self.uploader, podcast=self.podcast,
            since=since, fields=fields,sort=sort,
            output=output, rows=PAGESIZE, page=page);
        info!("url: {}", url);
        let query_pairs = vec![
            ("q", q.as_str()),
            ("sort[]", sort),
            ("output", output),
        ];
        let url = format!("{BASE_URL}/advancedsearch.php");
        match ureq::get(&url)
            .query_pairs(query_pairs)
            .set("Accept", "application/json")
            .call(){
            Ok(response) => {
                if response.status() == 200 {
                    match response.into_json::<serde_json::Value>(){
                        Ok(json) => {
                            let response = &json["response"];
                            let num_found = response["numFound"].as_u64().unwrap();
                            let start = response["start"].as_u64().unwrap();
                            debug!("Page: {}", page);
                            debug!("Start: {}", start);
                            debug!("Found: {}", num_found);
                            let pagesize: u64 = PAGESIZE.try_into().unwrap();
                            if num_found > start + pagesize {
                                debug!("Recursion");
                                let new_page = page + 1;
                                debug!("Page: {}", new_page);
                                let mut more_items = self.get_docs(since, new_page);
                                items.append(&mut more_items)
                            }
                            for (i, doc) in response["docs"].as_array().unwrap().iter().enumerate(){
                                debug!("Doc: {:?}", doc);
                                debug!("=============");
                                let mut doc: Doc = match serde_json::from_value(doc.clone()){
                                    Ok(doc) => {
                                        debug!("Got doc");
                                        doc
                                    },
                                    Err(e) => {
                                        error!("Error: {e}");
                                        continue
                                    },
                                };
                                let number = i + 1 + (page - 1) * PAGESIZE;
                                debug!("Doc {}. Number: {} => {}", doc.get_identifier(), i, number);
                                //debug!("Doc {:?}", &doc);
                                doc.set_number(number);
                                items.push(doc);
                            }
                        },
                        Err(e) => {
                            error!("Cant get response: {e}");
                        }
                    }
                }else{
                    let status_code = response.status();
                    let message = response.into_string().unwrap();
                    error!("HTTP Error: {status_code}. Error: {message}");
                }
            },
            Err(e) => {
                error!("Cant get response: {e}");
            }
        }
        items.sort_by_key(|b| std::cmp::Reverse(b.get_datetime()));
        items
    }

    pub fn get_mp3_metadata(identifier: &str) -> Option<Mp3Metadata>{
        let url = format!("{}/download/{identifier}/{identifier}_files.xml",
            BASE_URL, identifier=identifier);
        info!("url: {}", url);
        match ureq::get(&url)
            .call(){
            Ok(response) => {
                if response.status() == 200{
                    match response.into_string(){
                        Ok(content) => Mp3Metadata::new(&content),
                        Err(e) => {
                            error!("Cant convert response: {e}");
                            None
                        },
                    }
                }else{
                    let status_code = response.status();
                    let message = response.into_string().unwrap();
                    error!("HTTP Error: {status_code}. Error: {message}");
                    None
                }
            }
            Err(e) => {
                error!("Cant get response: {e}");
                None
            }
        }
    }

    pub fn get_metadata(identifier: &str) -> Option<IAMetadata>{
        let url = format!("{}/download/{identifier}/{identifier}_meta.xml",
            BASE_URL, identifier=identifier);
        info!("url: {}", url);
        match ureq::get(&url)
            .call(){
            Ok(response) => {
                if response.status() == 200{
                    match response.into_string(){
                        Ok(content) => Some(IAMetadata::new(&content)),
                        Err(e) => {
                            error!("Cant convert response: {e}");
                            None
                        },
                    }
                }else{
                    let status_code = response.status();
                    let message = response.into_string().unwrap();
                    error!("HTTP Error: {status_code}. Error: {message}");
                    None
                }
            }
            Err(e) => {
                error!("Cant get response: {e}");
                None
            }
        }
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
    use super::IAClient;


    #[test]
    fn test_get_docs(){
        tracing_subscriber::registry()
            .with(EnvFilter::from_str("debug").unwrap())
            .with(tracing_subscriber::fmt::layer())
            .init();

        let iaclient = IAClient::new( "atareao", "prueba");
        let docs = iaclient.get_docs("1970-01-01", 1);
        if !docs.is_empty(){
            debug!("{:?}", docs.first().unwrap());
        }
        assert!(!docs.is_empty())
    }
}
