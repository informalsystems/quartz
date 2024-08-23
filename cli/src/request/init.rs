use std::path::PathBuf;

use crate::{error::Error, request::Request};

#[derive(Clone, Debug)]
pub struct InitRequest {
    pub name: PathBuf,
}

impl TryFrom<InitRequest> for Request {
    type Error = Error;

    fn try_from(request: InitRequest) -> Result<Request, Error> {
        if request.name.extension().is_some() {
            return Err(Error::PathNotDir(format!("{}", request.name.display())));
        } else if request.name.exists() {
            return Err(Error::GenericErr(format!(
                "Directory already exists: {}",
                request.name.display()
            )));
        }

        Ok(Request::Init(request))
    }
}
