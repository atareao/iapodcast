use tracing::{info, debug, error};

pub struct Telegram{
    access_token: String,
    chat_id: String,
}

pub fn get_telegram_client() -> Option<Telegram>{
    match std::env::var("TELEGRAM_TOKEN"){
        Ok(token) => {
            match std::env::var("TELEGRAM_CHAT_ID"){
                Ok(chat_id) => Some(Telegram::new(&token, &chat_id)),
                Err(_) => None,
            }
        },
        Err(_) => None,
    }
}

impl Telegram{
    pub fn new(access_token: &str, chat_id: &str) -> Self{
        Self{
            access_token: access_token.to_string(),
            chat_id: chat_id.to_string(),
        }
    }

    pub fn send_audio(&self, audio: &str, caption: &str){
        let url = format!("https://api.telegram.org/bot{}/sendAudio",
            self.access_token);
        info!("url  {}", url);
        let content = Self::prepare(caption);

        match ureq::post(&url)
            .set("Content-Type", "application/json")
            .set("Accept", "application/json")
            .send_json(ureq::json!({
                "chat_id": self.chat_id,
                "audio": audio,
                "caption": content,
                "parse_mode": "HTML",
            })){
            Ok(response) => {
                if response.status() == 200 {
                    info!("Send audio");
                    debug!("Audio: {audio}");
                }else{
                    let status_code = response.status();
                    let error = response.into_string().unwrap();
                    error!("Error sending audio. HTTP Error: {status_code}. {error}")
                }
            },
            Err(e) => {
                error!("Error sending audio: {e}")
            }
        }
    }

    fn prepare(text: &str) -> String{
        text.chars()
            .map(|c| match c {
                '"' => '\'',
                _   => c,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use dotenv::dotenv;
    use std::env;
    use super::Telegram;

    #[test]
    fn send_audio_test(){
        dotenv().ok();
        let token = env::var("TOKEN").unwrap();
        let chat_id = env::var("CHAT_ID").unwrap();
        let audio = env::var("AUDIO").unwrap();
        let caption = r#"Buenas muchachada, he compartido un nuevo episodio <strong>Papá Friki 3 Wireguard2</strong>.
<a href="https://feeds.feedburner.com/papafriki">https://feeds.feedburner.com/papafriki</a> 
<a href="/papa-friki-3-wireguard2">Papá Friki 3 Wireguard2</a>
Ya sabéis, poco a poco irá llegando a vuestro programa de podcast favorito, a la red de SOSPECHOSOS HABITUALES, a Telegram o a YouTube"#;
        println!("==============================================");
        println!("{}, {}, {}, {}", token, chat_id, audio, caption);
        println!("==============================================");
        
        let telegram = Telegram::new(&token, &chat_id);
        telegram.send_audio(&audio, caption);
    }
}

