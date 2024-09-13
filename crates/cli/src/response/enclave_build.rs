use serde::Serialize;

use crate::response::Response;

#[derive(Clone, Debug, Serialize)]
pub struct EnclaveBuildResponse;

impl From<EnclaveBuildResponse> for Response {
    fn from(response: EnclaveBuildResponse) -> Self {
        Self::EnclaveBuild(response)
    }
}
