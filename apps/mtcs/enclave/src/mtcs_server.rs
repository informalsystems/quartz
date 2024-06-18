use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use cosmrs::{tendermint::account::Id as TmAccountId, AccountId};
use cosmwasm_std::HexBinary;
use cw_tee_mtcs::{
    msg::execute::SubmitSetoffsMsg,
    state::{RawCipherText, RawHash, SettleOff, Transfer},
};
use cycles_sync::types::RawObligation;
use ecies::{decrypt, encrypt};
use k256::ecdsa::{SigningKey, VerifyingKey};
use mtcs::{
    algo::mcmf::primal_dual::PrimalDual, impls::complex_id::ComplexIdMtcs,
    obligation::SimpleObligation, prelude::DefaultMtcs, setoff::SimpleSetoff, Mtcs,
};
use quartz_enclave::{
    attestor::Attestor,
    proto::{clearing_server::Clearing, RunClearingRequest, RunClearingResponse},
};
use serde::{Deserialize, Serialize};
use tonic::{Request, Response, Result as TonicResult, Status};

#[derive(Clone, Debug)]
pub struct MtcsService<A> {
    sk: Arc<Mutex<Option<SigningKey>>>,
    _attestor: A,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunClearingMessage {
    intents: BTreeMap<RawHash, RawCipherText>,
    liquidity_sources: Vec<HexBinary>,
}

impl<A> MtcsService<A>
where
    A: Attestor,
{
    pub fn new(sk: Arc<Mutex<Option<SigningKey>>>, _attestor: A) -> Self {
        Self { sk, _attestor }
    }
}

#[tonic::async_trait]
impl<A> Clearing for MtcsService<A>
where
    A: Attestor + Send + Sync + 'static,
{
    async fn run(
        &self,
        request: Request<RunClearingRequest>,
    ) -> TonicResult<Response<RunClearingResponse>> {
        let message: RunClearingMessage = {
            let message = request.into_inner().message;
            serde_json::from_str(&message).map_err(|e| Status::invalid_argument(e.to_string()))?
        };

        let digests_ciphertexts = message.intents;
        let (digests, ciphertexts): (Vec<_>, Vec<_>) = digests_ciphertexts.into_iter().unzip();

        let sk = self.sk.lock().unwrap();
        let obligations: Vec<SimpleObligation<_, i64>> = ciphertexts
            .into_iter()
            .map(|ciphertext| decrypt_obligation(sk.as_ref().unwrap(), &ciphertext))
            .collect();

        let mut mtcs = ComplexIdMtcs::wrapping(DefaultMtcs::new(PrimalDual::default()));
        let setoffs: Vec<SimpleSetoff<_, i64>> = mtcs.run(obligations).unwrap();

        let liquidity_sources: Vec<_> = message
            .liquidity_sources
            .into_iter()
            .map(|ls| VerifyingKey::from_sec1_bytes(&ls))
            .collect::<Result<_, _>>()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let setoffs_enc: BTreeMap<RawHash, SettleOff> = setoffs
            .into_iter()
            .map(|so| into_settle_offs(so, &liquidity_sources))
            .zip(digests)
            .map(|(settle_off, digest)| (digest, settle_off))
            .collect();

        let message = serde_json::to_string(&SubmitSetoffsMsg { setoffs_enc }).unwrap();
        Ok(Response::new(RunClearingResponse { message }))
    }
}

fn into_settle_offs(
    so: SimpleSetoff<HexBinary, i64>,
    liquidity_sources: &[VerifyingKey],
) -> SettleOff {
    let debtor_pk = VerifyingKey::from_sec1_bytes(&so.debtor).unwrap();
    let creditor_pk = VerifyingKey::from_sec1_bytes(&so.creditor).unwrap();

    if let Some(ls_pk) = liquidity_sources.iter().find(|ls| ls == &&debtor_pk) {
        // A setoff on a tender should result in the creditor's (i.e. the tender receiver) balance
        // decreasing by the setoff amount
        SettleOff::Transfer(Transfer {
            payer: wasm_address(creditor_pk),
            payee: wasm_address(*ls_pk),
            amount: so.set_off as u64,
        })
    } else if let Some(ls_pk) = liquidity_sources.iter().find(|ls| ls == &&creditor_pk) {
        // A setoff on an acceptance should result in the debtor's (i.e. the acceptance initiator)
        // balance increasing by the setoff amount
        SettleOff::Transfer(Transfer {
            payer: wasm_address(*ls_pk),
            payee: wasm_address(debtor_pk),
            amount: so.set_off as u64,
        })
    } else {
        SettleOff::SetOff(encrypt_setoff(so, debtor_pk, creditor_pk))
    }
}

fn wasm_address(pk: VerifyingKey) -> String {
    let tm_pk = TmAccountId::from(pk);
    AccountId::new("wasm", tm_pk.as_bytes())
        .unwrap()
        .to_string()
}

fn encrypt_setoff(
    so: SimpleSetoff<HexBinary, i64>,
    debtor_pk: VerifyingKey,
    creditor_pk: VerifyingKey,
) -> Vec<RawCipherText> {
    let so_ser = serde_json::to_string(&so).expect("infallible serializer");
    let so_debtor = encrypt(&debtor_pk.to_sec1_bytes(), so_ser.as_bytes()).unwrap();
    let so_creditor = encrypt(&creditor_pk.to_sec1_bytes(), so_ser.as_bytes()).unwrap();

    vec![so_debtor.into(), so_creditor.into()]
}

fn decrypt_obligation(
    sk: &SigningKey,
    ciphertext: &RawCipherText,
) -> SimpleObligation<HexBinary, i64> {
    let o: RawObligation = {
        let o = decrypt(&sk.to_bytes(), ciphertext).unwrap();
        serde_json::from_slice(&o).unwrap()
    };
    SimpleObligation::new(None, o.debtor, o.creditor, i64::try_from(o.amount).unwrap()).unwrap()
}
