use std::{env::current_dir, path::PathBuf};

use crate::{error::Error, request::Request};

#[derive(Clone, Debug)]
pub struct InitRequest {
    pub directory: PathBuf,
}

impl TryFrom<Option<PathBuf>> for InitRequest {
    type Error = Error;

    fn try_from(path: Option<PathBuf>) -> Result<Self, Self::Error> {
        if let Some(path) = path {
            Ok(InitRequest {directory: path})
        } else {
            Ok(InitRequest {directory: current_dir().map_err(|e| Error::GenericErr(e.to_string()))?})
        }
    }
}

impl From<InitRequest> for Request {
    fn from(request: InitRequest) -> Self {
        Self::Init(request)
    }
}
