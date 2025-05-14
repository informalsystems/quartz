use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{
    error::Error,
    handler::Handler,
    msg::execute::signed::{Signed, Verifier},
    state::SESSION,
};

impl<M, S> Handler for Signed<M, S>
where
    M: Handler + AsRef<[u8]>,
    S: Verifier,
{
    fn handle(
        self,
        mut deps: DepsMut<'_>,
        env: &Env,
        info: &MessageInfo,
    ) -> Result<Response, Error> {
        let session = SESSION.load(deps.storage).map_err(Error::Std)?;
        let pub_key = session.pub_key().ok_or(Error::MissingSessionPublicKey)?;
        let (msg, sig) = self.into_tuple();
        sig.verify(pub_key, &msg)?;
        Handler::handle(msg, deps.branch(), env, info)
    }
}
