use serde::Serialize;

use crate::response::Response;

#[derive(Clone, Debug, Serialize, Default)]
pub struct ListenResponse;

impl From<ListenResponse> for Response {
    fn from(response: ListenResponse) -> Self {
        Self::Listen(response)
    }
}
