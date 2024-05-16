// Copyright (c) 2018-2022 The MobileCoin Foundation

//! Attestation Verification Report type.

use core::fmt::Debug;

use mc_sgx_core_types::QuoteNonce;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Structure for holding the contents of the Enclave's Report Data.
/// The Enclave Quote's ReportData member contains a SHA256 hash of this
/// structure's contents.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct EnclaveReportDataContents {
    nonce: QuoteNonce,
    custom_identity: Option<[u8; 32]>,
}

impl EnclaveReportDataContents {
    /// Create a new EnclaveReportDataContents.
    ///
    /// # Arguments
    /// * `nonce` - The nonce provided from the enclave when generating the
    ///   Report.
    /// * `key` - The public key of the enclave. Previously this was bytes 0..32
    ///   of the enclave's [`ReportData`](mc-sgx-core-types::ReportData).
    /// * `custom_identity` - The custom identity of the enclave. Previously
    ///   this was bytes 32..64 of the enclave's
    ///   [`ReportData`](mc-sgx-core-types::ReportData).
    pub fn new(nonce: QuoteNonce, custom_identity: impl Into<Option<[u8; 32]>>) -> Self {
        Self {
            nonce,
            custom_identity: custom_identity.into(),
        }
    }

    /// Get the nonce
    pub fn nonce(&self) -> &QuoteNonce {
        &self.nonce
    }

    ///  Get the custom identity
    pub fn custom_identity(&self) -> Option<&[u8; 32]> {
        self.custom_identity.as_ref()
    }

    /// Returns a SHA256 hash of the contents of this structure.
    ///
    /// This is the value that is stored in bytes 0..32 of the enclave's
    /// [`ReportData`](mc-sgx-core-types::ReportData).
    pub fn sha256(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.nonce);
        if let Some(custom_identity) = &self.custom_identity {
            hasher.update(custom_identity);
        }
        hasher.finalize().into()
    }
}
