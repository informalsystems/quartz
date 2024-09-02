use crate::request::Request;

#[derive(Clone, Debug)]
pub struct EnclaveBuildRequest {}

impl From<EnclaveBuildRequest> for Request {
    fn from(request: EnclaveBuildRequest) -> Self {
        Self::EnclaveBuild(request)
    }
}
