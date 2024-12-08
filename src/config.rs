#[cfg(feature = "ssr")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ssr", derive(Serialize, Deserialize))]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub queue_size: usize,
}

impl Default for ServerConfig {
    // default to 0.0.0.0:1145
    fn default() -> Self {
        Self {
            address: "0.0.0.0".to_string(),
            port: 1145,
            queue_size: 32,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ssr", derive(Serialize, Deserialize))]
pub struct RssConfig {
    pub rsshub_url: String,
    pub default_interval: u64,
}

impl Default for RssConfig {
    // default to https://rsshub.app/
    fn default() -> Self {
        Self {
            rsshub_url: "https://rsshub.app/".to_string(),
            default_interval: 3600,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ssr", derive(Serialize, Deserialize))]
pub struct PushConfig {
    pub default_interval: u64,
}

impl Default for PushConfig {
    fn default() -> Self {
        Self {
            default_interval: 3600,
        }
    }
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "ssr", derive(Serialize, Deserialize))]
pub struct Config {
    pub server: ServerConfig,
    pub rss: RssConfig,
    pub push: PushConfig,
}

pub fn rss_hub_url_transform(url: &str) -> String {
    let mut result = String::new();
    if url.starts_with("http://") || url.starts_with("https://") {
        result.push_str(url);
    } else {
        result.push_str("https://");
        result.push_str(url);
    }
    if !url.ends_with("/") {
        result.push('/');
    }
    result
}

#[cfg(feature = "ssr")]
pub use server::*;

#[cfg(feature = "ssr")]
mod server {
    use anyhow::Context;
    use figment::{
        providers::{Env, Format, Serialized, Toml},
        Figment,
    };

    impl super::Config {
        pub fn load() -> Result<Self, figment::Error> {
            Figment::from(Serialized::defaults(super::Config::default()))
                .merge(Toml::file("pele.toml"))
                .merge(Env::prefixed("PELE_"))
                .extract()
        }

        pub fn save(&self) -> anyhow::Result<()> {
            let toml = toml::to_string(self).context("Failed to serialize config.")?;
            std::fs::write("pele.toml", toml).context("Failed to write config to disk.")?;
            Ok(())
        }
    }
}
