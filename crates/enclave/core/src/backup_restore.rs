use std::fmt::Debug;

/// Rudimentary backup and restore functionality
#[async_trait::async_trait]
pub trait Backup {
    /// The backup Config type specified by `Host`. (may include sealed file location)
    type Config;
    /// The error type returned by backup and restore ops.
    type Error;

    /// Persist the current state based on the specified config.
    /// Ideally implemented as a bunch of exports (see `Export` trait).
    /// Must panic on failure.
    async fn backup(&self, config: Self::Config);

    /// Restore the backed-up state based on the specified config.
    /// Ideally implemented as a bunch of imports (see `Import` trait).
    async fn try_restore(&mut self, config: Self::Config) -> Result<(), Self::Error>;
}

/// Export a type (and its contained data) as bytes. Analogous to serialization.
/// `Export` and `Import` implementations must be bijective.
/// Must panic on failure.
#[async_trait::async_trait]
pub trait Export {
    /// Export `self` (and the data it represents) as bytes
    async fn export(&self) -> Vec<u8>;
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
