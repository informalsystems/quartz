#[async_trait::async_trait]
pub trait Backup {
    /// The backup Config type specified by `Host`. (may include sealed file location)
    type Config;
    /// The error type returned by backup and restore ops.
    type Error;

    /// Persist the current state based on the specified config.
    /// Must panic on failure.
    async fn backup(&self, config: Self::Config);

    /// Restore the backed-up state based on the specified config.
    /// Must return `Ok(false)` if previous backup did not exist.
    fn try_restore(&self, config: Self::Config) -> Result<bool, Self::Error>;
}

#[async_trait::async_trait]
pub trait Export {
    async fn export(&self) -> Vec<u8>;
}

#[async_trait::async_trait]
pub trait Import {
    async fn import(self, data: Vec<u8>);
}
