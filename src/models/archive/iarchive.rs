use tracing::{debug, error};
use serde::{Serialize, Deserialize};
use super::{
    Doc,
    BASE_URL,
};

const PAGESIZE: usize = 200;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IAClient{
    pub uploader: String,
    pub podcast: String,
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
        let q: String = [
            format!("uploader:({uploader})", uploader=self.uploader),
            format!("publicdate:[{since} TO 9999-12-31]"),
            format!("podcast:({podcast})", podcast=self.podcast),
            "mediatype:(audio)".to_string(),
        ].join(" AND ");
        let page_str = page.to_string();
        let pagesize_str = PAGESIZE.to_string();
        let query_pairs = vec![
            ("q", q.as_str()),
            ("sort[]", "publicdate asc"),
            ("fl[]", "description"),
            ("fl[]", "downloads"),
            ("fl[]", "identifier"),
            ("fl[]", "item_size"),
            ("fl[]", "name"),
            ("fl[]", "publicdate"),
            ("fl[]", "publisher"),
            ("fl[]", "subject"),
            ("fl[]", "title"),
            ("output", "json"),
            ("rows", pagesize_str.as_str()),
            ("page", page_str.as_str()),
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
