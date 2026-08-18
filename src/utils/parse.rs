use crate::config::SYSCONFDIR;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConfigType {
    Xinux,
    #[default]
    Flakes,
    Legacy,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct BrandingConfig {
    pub slides: Vec<Slide>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Slide {
    pub title: String,
    pub subtitle: String,
    pub image: String,
}

pub fn parse_branding(brand: &str) -> Result<BrandingConfig> {
    let f = fs::read_to_string(format!(
        "{}/xeonitte/branding/{}/slides.yml",
        SYSCONFDIR, brand
    ))?;
    let config: BrandingConfig = serde_yaml::from_str(&f)?;
    Ok(config)
}
