#[cfg(feature = "ssr")]
use chrono::DateTime;
#[cfg(feature = "ssr")]
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
#[cfg(feature = "ssr")]
use tracing::warn;

#[cfg(feature = "ssr")]
use rss::{Channel, Item};
#[cfg(feature = "ssr")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "ssr", derive(Serialize, Deserialize))]
pub struct Subscription {
    pub url: String,
    pub name: String,
    pub interval: u64,
    pub last_update: u64,
    pub push_targets: Vec<usize>,
    // TODO: template
}

#[cfg(feature = "ssr")]
pub fn load_subscriptions() -> Result<Vec<Subscription>, figment::Error> {
    Figment::from(Serialized::defaults(Vec::<Subscription>::default()))
        .merge(Toml::file("subscriptions.toml"))
        .extract()
}

#[cfg(feature = "ssr")]
impl Subscription {
    pub async fn fetch(&mut self) -> anyhow::Result<Option<Vec<Item>>> {
        use anyhow::Context;
        let mut result = Vec::new();
        let content = reqwest::get(&self.url)
            .await
            .context("Failed to send request.")?
            .bytes()
            .await
            .context("Failed to decode bytes from response.")?;
        let channel =
            Channel::read_from(&content[..]).context("Failed to parse RSS channel data.")?;
        for item in channel.items() {
            if let Some(pub_time) = item.pub_date() {
                if let Ok(pub_time) = DateTime::parse_from_rfc2822(pub_time) {
                    if pub_time.timestamp() as u64 > self.last_update {
                        self.last_update = pub_time.timestamp() as u64;
                        result.push(item.clone());
                    }
                } else {
                    warn!("Failed to parse datetime: {}", pub_time);
                    continue;
                }
            } else {
                continue;
            }
        }
        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }
}
