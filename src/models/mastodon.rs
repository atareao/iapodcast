use reqwest::Client;
use serde_json::json;
use tracing::{info, error};

pub fn get_mastodon_client() -> Option<Mastodon>{
    match std::env::var("MASTODON_TOKEN"){
        Ok(token) => {
            match std::env::var("MASTODON_INSTANCE"){
                Ok(instance) => Some(Mastodon::new(&token, &instance)),
                Err(_) => None,
            }
        },
        Err(_) => None,
    }
}

pub struct Mastodon{
    instance: String,
    access_token: String,
}

impl Mastodon{
    pub fn new(access_token: &str, instance: &str) -> Self{
        Mastodon {
            instance: instance.to_string(),
            access_token: access_token.to_string(),
        }
    }

    pub async fn post(&self, message: &str){
        let url = format!("https://{}/api/v1/statuses", self.instance);
        info!("{}", &url);
        let body = json!({"status": message});
        match Client::new()
            .post(&url)
            .json(&body)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await{
                Ok(response) => {
                    info!("Mensaje envíado a Mastodon: {}",
                        response.status().to_string());
                },
                Err(error) => {
                    error!("No he podido enviar el mensaje a Mastodon: {}",
                        error.to_string());
                },
            }
    }
}
