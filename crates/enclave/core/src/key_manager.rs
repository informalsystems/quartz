pub mod default;
pub mod shared;

/// A trait defining the public key management functionality within the enclave.
///
/// The [`KeyManager`] trait is responsible for exposing the enclave's public key, which is stored
/// on-chain as part of the handshake. This can be used by users to encrypt their requests so that
/// they're only seen by the enclave.
///
/// The public key is expected to be convertible into a byte vector (`Vec<u8>`) for easy
/// serialization and on-chain storage.
#[async_trait::async_trait]
pub trait KeyManager: Send + Sync + 'static {
    /// The public key type for the KeyManager.
    ///
    /// This type must be convertible into a vector of bytes (`Vec<u8>`).
    type PubKey: Into<Vec<u8>>;

    /// Returns the enclave public key.
    async fn pub_key(&self) -> Self::PubKey;
}
