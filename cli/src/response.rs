use serde::Serialize;

use crate::response::init::InitResponse;

pub mod init;

#[derive(Clone, Debug, Serialize)]
pub enum Response {
    Init(InitResponse),
}
