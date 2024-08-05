use displaydoc::Display;
use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum Error {
    /// specified path `{0}` is not a directory
    PathNotDir(String),
    /// {0}
    GenericErr(String),
}
