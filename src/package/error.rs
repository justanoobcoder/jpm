use thiserror::Error;

pub const MAX_NAME_LEN: usize = 64;

#[derive(Error, Debug)]
pub enum FieldError {
    #[error("`{0}` must not be empty")]
    Empty(&'static str),
    #[error("`{0}` must not have leading or trailing whitespace")]
    Whitespace(&'static str),
}

#[derive(Error, Debug)]
pub enum NameError {
    #[error("package name is too long (max {MAX_NAME_LEN} characters)")]
    TooLong,
}

#[derive(Error, Debug)]
pub enum PackageError {
    #[error("Failed to parse manifest:\n{0}")]
    Parse(#[from] toml::de::Error),

    #[error(transparent)]
    Field(#[from] FieldError),

    #[error("Invalid package name: {0}")]
    InvalidName(#[from] NameError),
}
