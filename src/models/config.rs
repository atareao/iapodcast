use serde::{Serialize, Deserialize};
use tokio::fs::read_to_string;
use std::{process, fmt::{self, Display}};

use super::{site::Site, archive::ArchiveOrg};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Configuration{
    log_level: String,
    data: String,
    public: String,
    style_css: String,
    archiveorg: ArchiveOrg,
    site: Site,
}

impl Display for Configuration{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "log_level: {}\ndata: {}\npublic: {}",
            self.get_log_level(),
            self.get_data(),
            self.get_public(),
        )
    }
}

impl Configuration {
    pub fn get_site(&self) -> &Site {
        &self.site
    }

    pub fn get_archiveorg(&self) -> &ArchiveOrg{
        &self.archiveorg
    }

    pub fn get_log_level(&self) -> &str{
        &self.log_level
    }

    pub fn get_data(&self) -> &str{
        &self.data
    }

    pub fn get_public(&self) -> &str{
        &self.public
    }

    pub fn get_style_css(&self) -> &str{
        &self.style_css
    }

    pub async fn read_configuration() -> Configuration{
        let content = match read_to_string("config.yml")
            .await {
                Ok(value) => value,
                Err(e) => {
                    println!("Error with config file `config.yml`: {e}");
                    process::exit(0);
                }
            };
        match serde_yaml::from_str(&content){
            Ok(configuration) => configuration,
            Err(e) => {
                println!("Error with config file `config.yml`: {e}");
                process::exit(0);
            }
        }
    }
}

