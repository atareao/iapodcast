use tracing::{info, error, debug};

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

    pub fn post(&self, message: &str){
        let url = format!("https://{}/api/v1/statuses", self.instance);
        info!("{}", &url);
        match ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", self.access_token))
            .set("Content-Type", "application/json")
            .set("Accept", "application/json")
            .send_json(ureq::json!({
                "status": message,
            })){
                Ok(response) => {
                    if response.status() == 200{
                        info!("Send status");
                        debug!("Status: {message}");
                    }else{
                        let status_code = response.status();
                        let error = response.into_string().unwrap();
                        error!("Error sending status. HTTP Error: {status_code}. {error}")
                    }
                },
                Err(error) => {
                    error!("Error sending status: {error}")
                },
            }
    }
}

#[cfg(test)]
mod tests {
    use dotenv::dotenv;
    use super::get_mastodon_client;

    #[test]
    fn send_status_test(){
        dotenv().ok();
        if let Some(mastodon) = get_mastodon_client(){
            mastodon.post("Esto es una prueba")
        }
    }
}

