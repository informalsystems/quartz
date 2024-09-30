use tendermint_light_client::{
    components::io::{AtHeight, Io, IoError},
    types::LightBlock,
};

#[derive(Clone, Debug)]
pub struct NullIo;

impl Io for NullIo {
    fn fetch_light_block(&self, _height: AtHeight) -> Result<LightBlock, IoError> {
        unimplemented!("stateless verification does NOT need access to Io")
    }
}
