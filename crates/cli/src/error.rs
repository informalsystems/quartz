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
    /// Cache error: {0}
    Cache(String),
    /// Config error: {0}
    Config(String),
    /// TOML Error : {0}
    TomlError(#[from] toml::de::Error),
    /// TOML Error : {0}
    TomlSerError(#[from] toml::ser::Error),
    /// Tendermint error: {0}
    TendermintError(#[from] tendermint::Error),
    /// Clearscreen error: {0}
    ClearscreenError(#[from] clearscreen::Error),
    /// JSON Error: {0}
    JsonError(#[from] serde_json::Error),
    /// IO Error: {0}
    IoError(#[from] std::io::Error),
}
