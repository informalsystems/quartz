use serde::Serialize;

use crate::response::Response;

#[derive(Clone, Debug, Serialize, Default)]
pub struct HandshakeResponse;

impl From<HandshakeResponse> for Response {
    fn from(response: HandshakeResponse) -> Self {
        Self::Handshake(response)
    }
}
