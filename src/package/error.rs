use super::validator::MAX_NAME_LEN;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FieldError {
    #[error("`{0}` must not be empty")]
    Empty(&'static str),
    #[error("`{0}` must not have leading or trailing whitespace")]
    Whitespace(&'static str),
}

#[derive(Error, Debug)]
pub enum NameError {
    #[error("too long (max {MAX_NAME_LEN} characters)")]
    TooLong,
    #[error("invalid character `{0}` in name (allowed: a-z, A-Z, 0-9, `-`, `_`)")]
    InvalidChar(char),
    #[error("must not start with `{0}`")]
    InvalidStartChar(char),
}

#[derive(Error, Debug)]
pub enum PackageError {
    #[error("Failed to parse manifest:\n{0}")]
    Parse(#[from] toml::de::Error),

    #[error("Invalid field value: {0}")]
    Field(#[from] FieldError),

    #[error("Invalid package name: {0}")]
    InvalidName(#[from] NameError),
}
