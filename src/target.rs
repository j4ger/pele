#[cfg(feature = "ssr")]
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};

#[cfg(feature = "ssr")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "ssr", derive(Serialize, Deserialize))]
pub struct Target {
    pub id: usize,
    pub name: String,
    pub url: String,
    pub interval: u64,
}

#[cfg(feature = "ssr")]
pub fn load_targets() -> Result<Vec<Target>, figment::Error> {
    Figment::from(Serialized::defaults(Vec::<Target>::default()))
        .merge(Toml::file("targets.toml"))
        .extract()
}

#[cfg(feature = "ssr")]
pub async fn handle_target(target: Target) {}
