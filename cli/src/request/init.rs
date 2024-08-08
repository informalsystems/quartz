use std::path::Path;

use crate::{error::Error, request::Request};

#[derive(Clone, Debug)]
pub struct InitRequest {
    pub name: String,
}

impl TryFrom<InitRequest> for Request {
    type Error = Error;

    fn try_from(request: InitRequest) -> Result<Request, Error> {
        if Path::new(&request.name).iter().count() != 1 {
            return Err(Error::GenericErr("App name contains path".to_string()));
        }

        Ok(Request::Init(request))
    }
}
