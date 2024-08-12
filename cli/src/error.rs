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
}
