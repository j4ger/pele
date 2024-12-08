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

    pub fn load_targets() -> Result<Vec<super::Target>, figment::Error> {
        Figment::from(Serialized::defaults(Vec::<super::Target>::default()))
            .merge(Toml::file("targets.toml"))
            .extract()
    }

    pub fn save_targets(targets: &[super::Target]) -> anyhow::Result<()> {
        let toml = toml::to_string(targets).context("Failed to serialize targets.")?;
        std::fs::write("targets.toml", toml).context("Failed to write targets to disk.")?;
        Ok(())
    }
}
