use serde::Serialize;

use crate::response::Response;

#[derive(Clone, Debug, Serialize, Default)]
pub struct HandshakeResponse {
    pub pub_key: String,
}

impl From<HandshakeResponse> for Response {
    fn from(response: HandshakeResponse) -> Self {
        Self::Handshake(response)
    }
}
