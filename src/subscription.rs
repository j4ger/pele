#[derive(Clone, Debug)]
#[cfg_attr(feature = "ssr", derive(serde::Serialize, serde::Deserialize))]
pub struct Subscription {
    pub id: usize,
    pub url: String,
    pub name: String,
    pub interval: u64,
    pub last_update: u64,
    pub push_targets: Vec<usize>,
    // TODO: template
}

#[cfg(feature = "ssr")]
pub use server::*;

#[cfg(feature = "ssr")]
mod server {
    use anyhow::Context;
    use chrono::DateTime;
    use figment::{
        providers::{Format, Toml},
        Figment,
    };
    use rss::{Channel, Item};
    use tracing::warn;

    pub fn load_subscriptions() -> Vec<super::Subscription> {
        Figment::new()
            .merge(Toml::file("subscriptions.toml"))
            .extract()
            .unwrap_or(Vec::new()) // TODO: might be a source of error
    }

    pub fn save_subscriptions(subscriptions: &[super::Subscription]) -> anyhow::Result<()> {
        let toml = toml::to_string(subscriptions).context("Failed to serialize subscriptions.")?;
        std::fs::write("subscriptions.toml", toml)
            .context("Failed to write subscriptions to disk.")?;
        Ok(())
    }

    impl super::Subscription {
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
}
