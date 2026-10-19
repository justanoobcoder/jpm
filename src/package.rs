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
    pub pkgrel: u8,
    pub desc: String,
    pub license: String,
    pub url: Option<String>,
    pub src: String,
    pub hash: String,
    pub src_dir: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Dependency {
    pub build: Option<Vec<String>>,
    pub runtime: Option<Vec<String>>,
    pub optional: Option<Vec<String>>,
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

#[derive(Deserialize, Debug)]
pub struct Build {
    pub system: String, // "autotools", "cmake", "meson", "make", and "custom"
    pub configure: Option<String>,
    pub compile: Option<String>,
    pub install: Option<String>,
    pub out_of_src: Option<bool>,
}

pub fn parse(content: &str) -> Result<Package, toml::de::Error> {
    toml::from_str(content)
}
