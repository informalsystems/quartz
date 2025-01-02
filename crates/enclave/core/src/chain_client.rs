pub mod default;

pub trait ChainClient: Send + Sync + 'static {
    const CHAIN_ID: &'static str;

    type Contract;
}
