use serde::Serialize;

use crate::response::Response;

#[derive(Clone, Debug, Serialize)]
pub struct DevResponse;

impl From<DevResponse> for Response {
    fn from(response: DevResponse) -> Self {
        Self::Dev(response)
    }
}
