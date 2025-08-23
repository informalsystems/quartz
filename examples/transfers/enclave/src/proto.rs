#[derive(Clone, Debug, PartialEq)]
pub struct UpdateRequest {
    pub message: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpdateResponse {
    pub message: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct QueryRequest {
    pub message: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct QueryResponse {
    pub message: String,
}
