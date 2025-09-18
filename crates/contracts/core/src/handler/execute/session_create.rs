use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{
    error::Error,
    handler::Handler,
    msg::execute::session_create::SessionCreate,
    state::{Session, SESSION},
};

impl Handler for SessionCreate {
    // Create new SESSION with msg.nonce and no pubkey.
    fn handle(self, deps: DepsMut<'_>, env: &Env, _info: &MessageInfo) -> Result<Response, Error> {
        // TODO(hu55a1n1): overwrite previous session?

        // ASSERT msg.contract == env.contract.address
        let addr = deps.api.addr_validate(self.contract())?;
        if addr != env.contract.address {
            return Err(Error::ContractAddrMismatch);
        }

        // STORE in SESSION: (msg.nonce, None)
        SESSION
            .save(deps.storage, &Session::create(self.nonce()))
            .map_err(Error::Std)?;

        Ok(Response::new().add_attribute("action", "session_create"))
    }
}
