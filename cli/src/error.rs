use displaydoc::Display;
use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum Error {
    /// Specified path `{0}` is not a directory
    PathNotDir(String),
    /// Specified file `{0}` does not exist
    PathNotFile(String),
    /// unspecified error: {0}
    GenericErr(String),
    /// IoError: {0}
    IoError(String),
    /// TOML Error : {0}
    TomlError(String),
    /// Tendermint error: {0}
    TendermintError(String),
    /// Clearscreen error: {0}
    ClearscreenError(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::TomlError(err.to_string())
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Error::TomlError(err.to_string())
    }
}

impl From<tendermint::Error> for Error {
    fn from(err: tendermint::Error) -> Self {
        Error::TendermintError(err.to_string())
    }
}

impl From<clearscreen::Error> for Error {
    fn from(err: clearscreen::Error) -> Self {
        Error::ClearscreenError(err.to_string())
    }
}
