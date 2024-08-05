use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

pub type Fmspc = [u8; 6];

#[cw_serde]
pub struct TcbInfo {
    pub info: String,
    //  pub certificate: String,
}

pub const DATABASE: Map<Fmspc, TcbInfo> = Map::new("state");
pub const ROOT_CERTIFICATE: Item<String> = Item::new("root_certificate");
