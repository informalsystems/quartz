use core::fmt::Debug;
use core::marker::PhantomData;

use ibc_relayer_types::core::ics23_commitment::error::Error as ProofError;
use ics23::{
    calculate_existence_root, commitment_proof::Proof, verify_membership, CommitmentProof,
    ProofSpec,
};
use tendermint::merkle::proof::ProofOps;
use tendermint_rpc::endpoint::abci_query::AbciQuery;

// Copied from hermes
pub fn convert_tm_to_ics_merkle_proof(
    tm_proof: &ProofOps,
) -> Result<Vec<CommitmentProof>, ProofError> {
    let mut proofs = Vec::new();

    for op in &tm_proof.ops {
        let mut parsed = CommitmentProof { proof: None };

        prost::Message::merge(&mut parsed, op.data.as_slice())
            .map_err(ProofError::commitment_proof_decoding_failed)?;

        proofs.push(parsed);
    }

    Ok(proofs)
}

trait Verifier {
    type Proof;
    type Root: Eq;
    type Key;
    type Value;
    type Error;

    fn verify(
        &self,
        proof: &Self::Proof,
        key: &Self::Key,
        value: &Self::Value,
    ) -> Result<Self::Root, Self::Error>;

    fn verify_against_root(
        &self,
        proof: &Self::Proof,
        key: &Self::Key,
        value: &Self::Value,
        root: &Self::Root,
    ) -> Result<bool, Self::Error> {
        let found_root = self.verify(proof, key, value)?;
        Ok(root == &found_root)
    }
}

#[derive(Clone, Debug)]
struct Ics23MembershipVerifier<K, V> {
    spec: ProofSpec,
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> Ics23MembershipVerifier<K, V> {
    fn new(spec: ProofSpec) -> Self {
        Self {
            spec,
            _phantom: Default::default(),
        }
    }
}

impl<K, V> Verifier for Ics23MembershipVerifier<K, V>
where
    K: AsRef<[u8]>,
    V: AsRef<[u8]>,
{
    type Proof = CommitmentProof;
    type Root = Vec<u8>;
    type Key = K;
    type Value = V;
    type Error = ProofError;

    fn verify(
        &self,
        commitment_proof: &Self::Proof,
        key: &Self::Key,
        value: &Self::Value,
    ) -> Result<Self::Root, Self::Error> {
        if value.as_ref().is_empty() {
            return Err(ProofError::empty_verified_value());
        }

        let Some(Proof::Exist(existence_proof)) = &commitment_proof.proof else {
            return Err(ProofError::invalid_merkle_proof());
        };

        let root = calculate_existence_root::<ics23::HostFunctionsManager>(existence_proof)
            .map_err(|_| ProofError::invalid_merkle_proof())?;

        if !verify_membership::<ics23::HostFunctionsManager>(
            commitment_proof,
            &self.spec,
            &root,
            key.as_ref(),
            value.as_ref(),
        ) {
            return Err(ProofError::verification_failure());
        }

        Ok(root)
    }
}

#[derive(Clone, Debug)]
struct MultiVerifier<V, const N: usize> {
    verifiers: [V; N],
}

impl<V, const N: usize> MultiVerifier<V, N> {
    fn new(verifiers: [V; N]) -> Self {
        assert!(N > 0, "cannot create empty multi-verifiers");
        Self { verifiers }
    }
}

impl<V, const N: usize> Verifier for MultiVerifier<V, N>
where
    V: Verifier,
    V::Value: Clone,
    V::Root: Into<V::Value> + Clone,
{
    type Proof = [V::Proof; N];
    type Root = V::Root;
    type Key = [V::Key; N];
    type Value = V::Value;
    type Error = V::Error;

    fn verify(
        &self,
        proofs: &Self::Proof,
        keys: &Self::Key,
        value: &Self::Value,
    ) -> Result<Self::Root, Self::Error> {
        let mut root = None;
        let mut value = value.clone();

        for (idx, verifier) in self.verifiers.iter().enumerate() {
            let proof = &proofs[idx];
            let key = &keys[N - idx - 1];
            let sub_root = verifier.verify(proof, key, &value)?;

            value = sub_root.clone().into();
            root = Some(sub_root);
        }

        Ok(root.expect("MultiVerifier cannot be instantiated with 0 verifiers"))
    }
}

#[derive(Clone, Debug)]
pub struct CwVerifier(MultiVerifier<Ics23MembershipVerifier<Vec<u8>, Vec<u8>>, 2>);

impl CwVerifier {
    pub fn new() -> Self {
        let mv = MultiVerifier::new([
            Ics23MembershipVerifier::new(ics23::iavl_spec()),
            Ics23MembershipVerifier::new(ics23::tendermint_spec()),
        ]);
        Self(mv)
    }

    pub fn verify(
        &self,
        proofs: &[CommitmentProof; 2],
        root: &Vec<u8>,
        keys: &[Vec<u8>; 2],
        value: &Vec<u8>,
    ) -> Result<(), ProofError> {
        if root.is_empty() {
            return Err(ProofError::empty_merkle_root());
        }

        let verified = self.0.verify_against_root(proofs, keys, value, root)?;
        if !verified {
            return Err(ProofError::verification_failure());
        }
        Ok(())
    }
}

pub trait CwProof {
    type Key;
    type Value;
    type Proof;

    fn verify(self, root: Vec<u8>) -> Result<(), ProofError>;
}

trait IntoKeys {
    fn into_keys(self) -> Vec<Vec<u8>>;
}

struct PrefixedKey<P, K> {
    key: K,
    prefix: PhantomData<P>,
}

impl<P, K> PrefixedKey<P, K> {
    fn new(key: K) -> Self {
        Self {
            key,
            prefix: PhantomData,
        }
    }
}

impl<P, K> IntoKeys for PrefixedKey<P, K>
where
    K: Into<Vec<u8>>,
    P: ConstPrefix,
{
    fn into_keys(self) -> Vec<Vec<u8>> {
        vec![P::PREFIX.to_string().into_bytes(), self.key.into()]
    }
}

struct PrefixWasm;

trait ConstPrefix {
    const PREFIX: &'static str;
}

impl ConstPrefix for PrefixWasm {
    const PREFIX: &'static str = "wasm";
}

impl<K> IntoKeys for K
where
    K: Into<Vec<u8>>,
{
    fn into_keys(self) -> Vec<Vec<u8>> {
        vec![self.into()]
    }
}

pub struct QueryProof<K, V> {
    proof: ProofOps,
    key: PrefixedKey<PrefixWasm, K>,
    value: V,
}

pub type RawQueryProof = QueryProof<Vec<u8>, Vec<u8>>;

#[derive(Clone, Debug)]
pub struct ErrorWithoutProof;

impl TryFrom<AbciQuery> for RawQueryProof {
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

impl<K, V> CwProof for QueryProof<K, V>
where
    K: Into<Vec<u8>>,
    V: Into<Vec<u8>>,
{
    type Key = K;
    type Value = V;
    type Proof = ProofOps;

    fn verify(self, root: Vec<u8>) -> Result<(), ProofError> {
        fn as_array_2<T: Debug>(v: Vec<T>) -> Result<[T; 2], ProofError> {
            let boxed_slice = v.into_boxed_slice();
            let boxed_array: Box<[T; 2]> = boxed_slice.try_into().expect("todo");
            Ok(*boxed_array)
        }

        let Self { proof, key, value } = self;
        let proofs = convert_tm_to_ics_merkle_proof(&proof)?;

        let cw_verifier = CwVerifier::new();
        cw_verifier.verify(
            &as_array_2(proofs)?,
            &root,
            &as_array_2(key.into_keys())?,
            &value.into(),
        )?;

        Ok(())
    }
}
