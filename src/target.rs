#[derive(Clone, Debug)]
#[cfg_attr(feature = "ssr", derive(serde::Serialize, serde::Deserialize))]
pub struct Target {
    pub id: usize,
    pub name: String,
    pub url: String,
    pub interval: u64,
}

#[cfg(feature = "ssr")]
pub use server::*;

#[cfg(feature = "ssr")]
mod server {
    use anyhow::Context;
    use figment::{
        providers::{Format, Serialized, Toml},
        Figment,
    };

    pub fn load_targets() -> Vec<super::Target> {
        Figment::from(Serialized::defaults(Vec::<super::Target>::new()))
            .merge(Toml::file("targets.toml"))
            .extract()
            .unwrap_or(Vec::new())
    }

    pub fn save_targets(targets: &[super::Target]) -> anyhow::Result<()> {
        let toml = toml::to_string(targets).context("Failed to serialize targets.")?;
        std::fs::write("targets.toml", toml).context("Failed to write targets to disk.")?;
        Ok(())
    }
}
