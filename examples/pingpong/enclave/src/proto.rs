#[derive(Clone, Debug, PartialEq)]
pub struct PingRequest {
    pub message: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PongResponse {
    pub message: String,
}
