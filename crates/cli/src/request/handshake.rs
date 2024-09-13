use cosmrs::AccountId;

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct HandshakeRequest {
    pub contract: AccountId,
    pub use_latest_trusted: bool,
}

impl From<HandshakeRequest> for Request {
    fn from(request: HandshakeRequest) -> Self {
        Self::Handshake(request)
    }
}
