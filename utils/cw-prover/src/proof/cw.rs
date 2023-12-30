use core::fmt::Debug;

use ibc_relayer_types::core::ics23_commitment::error::Error as ProofError;
use tendermint::merkle::proof::ProofOps;
use tendermint_rpc::endpoint::abci_query::AbciQuery;

use crate::{
    proof::{
        convert_tm_to_ics_merkle_proof,
        key::{IntoKeys, PrefixedKey},
        prefix::PrefixWasm,
        Proof,
    },
    verifier::cw::CwVerifier,
};

pub type RawCwProof = CwProof<Vec<u8>, Vec<u8>>;

#[derive(Clone, Debug)]
pub struct CwProof<K, V> {
    proof: ProofOps,
    key: PrefixedKey<PrefixWasm, K>,
    value: V,
}

#[derive(Clone, Debug)]
pub struct ErrorWithoutProof;

impl TryFrom<AbciQuery> for RawCwProof {
    type Error = ErrorWithoutProof;

    fn try_from(query: AbciQuery) -> Result<Self, Self::Error> {
        let AbciQuery {
            key, value, proof, ..
        } = query;
        let Some(proof) = proof else {
            return Err(ErrorWithoutProof);
        };

        Ok(Self {
            proof,
            key: PrefixedKey::new(key),
            value,
        })
    }
}

impl<K, V> Proof for CwProof<K, V>
where
    K: Into<Vec<u8>>,
    V: Into<Vec<u8>>,
{
    type Key = K;
    type Value = V;
    type ProofOps = ProofOps;

    fn verify(self, root: Vec<u8>) -> Result<(), ProofError> {
        fn into_array_of_size_2<T: Debug>(v: Vec<T>) -> Result<[T; 2], ProofError> {
            let boxed_slice = v.into_boxed_slice();
            let boxed_array: Box<[T; 2]> = boxed_slice.try_into().expect("todo");
            Ok(*boxed_array)
        }

        let Self { proof, key, value } = self;
        let proofs = convert_tm_to_ics_merkle_proof(&proof)?;

        let cw_verifier = CwVerifier::new();
        cw_verifier.verify(
            &into_array_of_size_2(proofs)?,
            &root,
            &into_array_of_size_2(key.into_keys())?,
            &value.into(),
        )?;

        Ok(())
    }
}
