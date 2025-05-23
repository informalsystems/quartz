use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{
    error::Error,
    handler::Handler,
    msg::execute::signed::{Auth, MsgVerifier, Signed},
};

impl<M, A, P, S> Handler for Signed<M, A>
where
    M: Handler + MsgVerifier<PubKey = P, Sig = S>,
    A: Auth<P, S>,
{
    fn handle(
        self,
        mut deps: DepsMut<'_>,
        env: &Env,
        info: &MessageInfo,
    ) -> Result<Response, Error> {
        let (msg, auth) = self.into_tuple();
        let pub_key = auth.pub_key();
        msg.verify(pub_key, auth.sig())?;
        Handler::handle(msg, deps.branch(), env, info)
    }
}
