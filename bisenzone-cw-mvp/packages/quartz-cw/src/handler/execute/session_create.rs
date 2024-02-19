use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::error::Error;
use crate::handler::Handler;
use crate::msg::execute::session_create::SessionCreate;
use crate::state::{Session, SESSION};

impl Handler for SessionCreate {
    fn handle(self, deps: DepsMut<'_>, _env: &Env, _info: &MessageInfo) -> Result<Response, Error> {
        // TODO(hu55a1n1): overwrite previous session?
        SESSION
            .save(deps.storage, &Session::create(self.into_nonce()))
            .map_err(Error::Std)?;

        Ok(Response::new().add_attribute("action", "session_create"))
    }
}
