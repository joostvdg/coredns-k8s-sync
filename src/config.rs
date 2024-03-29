use serde::{Deserialize, Serialize};
use std::{error::Error, fmt, fs};
use log::{info};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    pub external_sources: Vec<ExternalSource>,
    pub destination_file_path: String,
    pub source_file_paths: Vec<String>,
    pub temp_storage_path: String,
    pub ttl: u64,
    pub call_frequency_in_minutes: u64,
    pub ca_cert_base64: String,
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            external_sources: vec![],
            destination_file_path: "".to_string(),
            temp_storage_path: "".to_string(),
            source_file_paths: vec![],
            ttl: 3600,
            call_frequency_in_minutes: 1,
            ca_cert_base64: "".to_string(),
            log_level: "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ExternalSource {
    pub url: String,
    pub domain_name: String,
    pub source_name: String,
}

pub fn load_config(config_file_path: String) -> std::result::Result<Config, Box<dyn Error>> {
    let config: Config = serde_json::from_str(&fs::read_to_string(config_file_path)?)?;
    Ok(config)
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "External Sources: {:?}\nDestination File Path: {}\nSource File Paths: {:?}\nTTL: {}\nCall Frequency: {}\nCA Cert Base64: {}\nLog Level: {}\n",
            self.external_sources, self.destination_file_path, self.source_file_paths, self.ttl, self.call_frequency_in_minutes, self.ca_cert_base64, self.log_level
        )
    }
}
impl fmt::Display for ExternalSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "URL: {}\nDomain Name: {}\nSource Name: {}\n",
            self.url, self.domain_name, self.source_name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_read_json() {
        let json = json!({
            "external_sources": [
                {
                    "url": "https://api.example.com",
                    "domain_name": "example.com",
                    "source_name": "example"
                }
            ],
            "destination_file_path": "/var/lib/coredns/db.home.lab",
            "temp_storage_path": "/tmp/coredns/",
            "source_file_paths": ["/path/to/source1"],
            "ttl": 3600,
            "call_frequency_in_minutes": 2,
            "ca_cert_base64": "base64-encoded-ca-cert",
            "log_level": "info"
        });

        let config: Config = serde_json::from_value(json).unwrap();

        assert_eq!(config.external_sources[0].url, "https://api.example.com");
        assert_eq!(config.external_sources[0].domain_name, "example.com");
        assert_eq!(config.external_sources[0].source_name, "example");
        assert_eq!(config.destination_file_path, "/var/lib/coredns/db.home.lab");
        assert_eq!(config.temp_storage_path, "/tmp/coredns/");
        assert_eq!(config.source_file_paths[0], "/path/to/source1");
        assert_eq!(config.ttl, 3600);
        assert_eq!(config.call_frequency_in_minutes, 2);
        assert_eq!(config.ca_cert_base64, "base64-encoded-ca-cert");
        assert_eq!(config.log_level, "info");
    }
}


// handle listening on a channel for config updates
pub async fn handle_config_update(mut receiver: tokio::sync::mpsc::Receiver<Result<notify::Event, notify::Error>>)  {
    info!("Listening for config updates...");
    while let Some(_) = receiver.recv().await {
        // reload the config
        let config = load_config("config.json".to_string()).unwrap();
        info!("Config updated: {}", config);
    }

}
