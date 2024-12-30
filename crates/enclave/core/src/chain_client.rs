pub trait ChainClient: Send + Sync {
    const CHAIN_ID: &'static str;

    type Contract;
}
