use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub info: Info,
    pub dependency: Option<Dependency>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub hook: Option<Hook>,
    pub build: Build,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Info {
    pub name: String,
    pub pkgver: String,
    pub pkgrel: u32,
    pub desc: String,
    pub license: String,
    pub url: Option<String>,
    pub src: String,
    pub hash: String,
    pub src_dir: Option<String>,
}

#[derive(Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct Dependency {
    #[serde(default)]
    pub build: Vec<String>,
    #[serde(default)]
    pub runtime: Vec<String>,
    #[serde(default)]
    pub optional: Vec<String>,
    #[serde(default)]
    pub conflicts: Vec<String>,
}

#[derive(Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct Hook {
    #[serde(default)]
    pub patches: Vec<String>,
    pub pre_configure: Option<String>,
    pub pre_install: Option<String>,
    pub post_install: Option<String>,
    pub pre_remove: Option<String>,
    pub post_remove: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BuildSystem {
    Autotools,
    Cmake,
    Meson,
    Make,
    Custom,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Build {
    pub system: BuildSystem,
    pub configure: Option<String>,
    pub compile: Option<String>,
    pub install: Option<String>,
    pub out_of_src: Option<bool>,
}
