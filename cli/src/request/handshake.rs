use std::path::PathBuf;

use cosmrs::{tendermint::chain::Id as ChainId, AccountId};

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct HandshakeRequest {
    pub contract: AccountId,
    pub port: u16,
    pub sender: String,
    pub chain_id: ChainId,
    pub node_url: String,
    pub rpc_addr: String,
    pub path: PathBuf
}

// TODO: Would I ever need this?
// impl TryFrom<HandshakeRequest> for HandshakeRequest {
//     type Error = Error;

//     fn try_from(path: Option<PathBuf>) -> Result<Self, Self::Error> {
//         if let Some(path) = path {
//             if !path.is_dir() {
//                 return Err(Error::PathNotDir(format!("{}", path.display())));
//             }
//         }

//         todo!()
//     }
// }

impl From<HandshakeRequest> for Request {
    fn from(request: HandshakeRequest) -> Self {
        Self::Handshake(request)
    }
}
