pub trait ChainClient {
    const CHAIN_ID: &'static str;

    type Contract;
}
