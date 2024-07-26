use displaydoc::Display;
use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum Error {
    /// Specified path `{0}` is not a directory
    PathNotDir(String),
    /// {0}
    GenericErr(String)
}
