use std::fmt::Debug;

/// Rudimentary backup and restore functionality
#[async_trait::async_trait]
pub trait Backup {
    /// The backup Config type specified by `Host`. (may include sealed file location)
    type Config;
    /// The error type returned by backup and restore ops.
    type Error: Send + Sync;

    /// Persist the current state based on the specified config.
    /// Ideally implemented as a bunch of exports (see `Export` trait).
    async fn backup(&self, config: Self::Config) -> Result<(), Self::Error>;

    /// Restore the backed-up state based on the specified config.
    /// Ideally implemented as a bunch of imports (see `Import` trait).
    async fn try_restore(&mut self, config: Self::Config) -> Result<(), Self::Error>;
}

/// Export a type (and its contained data) as bytes. Analogous to serialization.
/// `Export` and `Import` implementations must be bijective.
/// Must panic on failure.
#[async_trait::async_trait]
pub trait Export {
    /// The error type returned by export ops.
    type Error: Send + Sync + Debug;

    /// Export `self` (and the data it represents) as bytes
    async fn export(&self) -> Result<Vec<u8>, Self::Error>;
}

#[async_trait::async_trait]
impl Export for () {
    type Error = ();

    async fn export(&self) -> Result<Vec<u8>, Self::Error> {
        Ok(vec![])
    }
}

/// Import bytes as a type. Analogous to deserialization.
/// `Export` and `Import` implementations must be bijective.
#[async_trait::async_trait]
pub trait Import: Sized {
    /// The error type returned by import ops.
    type Error: Send + Sync + Debug;

    /// Import bytes as `Self`.
    async fn import(data: Vec<u8>) -> Result<Self, Self::Error>;
}

#[async_trait::async_trait]
impl Import for () {
    type Error = ();

    async fn import(_data: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(())
    }
}
