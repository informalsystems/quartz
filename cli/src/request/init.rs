use std::path::PathBuf;

use crate::{error::Error, request::Request};

#[derive(Clone, Debug)]
pub struct InitRequest {
    // TODO(hu55a1n1): remove `allow(unused)` here once init handler is implemented
    #[allow(unused)]
    directory: PathBuf,
}

impl TryFrom<Option<PathBuf>> for InitRequest {
    type Error = Error;

    fn try_from(path: Option<PathBuf>) -> Result<Self, Self::Error> {
        if let Some(path) = path {
            if !path.is_dir() {
                return Err(Error::PathNotDir(format!("{}", path.display())));
            }
        }

        todo!()
    }
}

impl From<InitRequest> for Request {
    fn from(request: InitRequest) -> Self {
        Self::Init(request)
    }
}
