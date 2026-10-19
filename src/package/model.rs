use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Package {
    pub info: Info,
    pub dependency: Option<Dependency>,
    pub hook: Option<Hook>,
    pub build: Build,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug, Default)]
pub struct Dependency {
    #[serde(default)]
    pub build: Vec<String>,
    #[serde(default)]
    pub runtime: Vec<String>,
    #[serde(default)]
    pub optional: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Hook {
    pub patches: Option<String>,
    pub pre_configure: Option<String>,
    pub pre_install: Option<String>,
    pub post_install: Option<String>,
    pub pre_remove: Option<String>,
    pub post_remove: Option<String>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BuildSystem {
    Autotools,
    Cmake,
    Meson,
    Make,
    Custom,
}

#[derive(Deserialize, Debug)]
pub struct Build {
    pub system: BuildSystem,
    pub configure: Option<String>,
    pub compile: Option<String>,
    pub install: Option<String>,
    pub out_of_src: Option<bool>,
}
