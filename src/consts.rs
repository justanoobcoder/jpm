pub const _APP_NAME: &str = "jpm";
pub const _DEFAULT_REPO_URL: &str = "https://github.com/justanoobcoder/jpm";
pub const MANIFEST_FILE: &str = "package.toml";

pub mod env {
    pub const MANIFEST_DIR_ENV: &str = "JPM_MANIFEST_DIR";
    pub const _SRC_CACHE_DIR_ENV: &str = "JPM_SRC_CACHE_DIR";
    pub const _BUILD_DIR_ENV: &str = "JPM_BUILD_DIR";
    pub const _STAGE_DIR_ENV: &str = "JPM_STAGE_DIR";
    pub const _BUILD_LOG_DIR_ENV: &str = "JPM_BUILD_LOG_DIR";
    pub const _ROOT_ENV: &str = "JPM_ROOT";

    pub const JPM_MANIFEST_DIR: &str = "/usr/share/jpm";
    pub const _JPM_SRC_CACHE_DIR: &str = "/var/cache/jpm/sources";
    pub const _JPM_BUILD_DIR: &str = "/tmp/jpm/build";
    pub const _JPM_STAGE_DIR: &str = "/tmp/jpm/stage";
    pub const _JPM_BUILD_LOG_DIR: &str = "/var/log/jpm";
    pub const _JPM_ROOT: &str = "";
}
