use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

use tendermint::Hash;
use tendermint_light_client::{
    builder::LightClientBuilder,
    components::{clock::SystemClock, scheduler},
    instance::Instance,
    light_client::Options,
    predicates::ProdPredicates,
    store::{memory::MemoryStore, LightStore},
    types::{Height, LightBlock, Status},
    verifier::ProdVerifier,
};

use crate::{error::Error, null_io::NullIo};

/// A interface over a stateless light client instance.
#[derive(Debug)]
pub struct StatelessProvider {
    #[allow(unused)]
    chain_id: String,
    instance: Instance,
}

impl StatelessProvider {
    pub fn new(chain_id: String, instance: Instance) -> Self {
        Self { chain_id, instance }
    }

    pub fn verify_to_height(&mut self, height: Height) -> Result<LightBlock, Error> {
        self.instance
            .light_client
            .verify_to_target(height, &mut self.instance.state)
            .map_err(Into::<Error>::into)
    }
}

pub fn make_provider(
    chain_id: &str,
    trusted_height: Height,
    trusted_hash: Hash,
    trace: Vec<LightBlock>,
    options: Options,
) -> Result<StatelessProvider, Error> {
    // Make sure the trace is not empty and that the first light block corresponds to trusted
    verify_trace_against_trusted(&trace, trusted_height, trusted_hash)?;

    let mut light_store = Box::new(MemoryStore::new());

    for light_block in &trace {
        light_store.insert(light_block.clone(), Status::Unverified);
    }

    let node_id = trace[0].provider;

    let instance = LightClientBuilder::custom(
        node_id,
        options,
        light_store,
        Box::new(NullIo {}),
        Box::new(SystemClock),
        #[allow(clippy::box_default)]
        Box::new(ProdVerifier::default()),
        Box::new(scheduler::basic_bisecting_schedule),
        Box::new(ProdPredicates),
    )
    .trust_light_block(trace[0].clone())
    .map_err(Into::<Error>::into)?
    .build();

    Ok(StatelessProvider::new(chain_id.to_string(), instance))
}

fn verify_trace_against_trusted(
    trace: &[LightBlock],
    trusted_height: Height,
    trusted_hash: Hash,
) -> Result<(), Error> {
    let Some(first_block) = trace.first() else {
        return Err(Error::EmptyTrace);
    };

    let first_height = first_block.signed_header.header.height;
    let first_hash = first_block.signed_header.header.hash();

    if first_height != trusted_height || first_hash != trusted_hash {
        return Err(Error::FirstTraceBlockNotTrusted {
            expected: (first_height, first_hash),
            found: (trusted_height, trusted_hash),
        });
    }

    Ok(())
}
