use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use cosmrs::{tendermint::account::Id as TmAccountId, AccountId};
use cosmwasm_std::{Addr, HexBinary};
//TODO: get rid of this
use cw_tee_mtcs::{
    msg::execute::SubmitSetoffsMsg,
    state::{RawHash, SettleOff, Transfer},
};
use cycles_sync::types::{ContractObligation, RawObligation};
use ecies::{decrypt, encrypt};
use k256::ecdsa::{SigningKey, VerifyingKey};
use mtcs::{
    algo::mcmf::primal_dual::PrimalDual, impls::complex_id::ComplexIdMtcs,
    obligation::SimpleObligation, prelude::DefaultMtcs, setoff::SimpleSetoff, Mtcs,
};
use quartz_cw::msg::execute::attested::RawAttested;
use quartz_enclave::attestor::Attestor;
use serde::{Deserialize, Serialize};
use tonic::{Request, Response, Result as TonicResult, Status};

use crate::proto::{clearing_server::Clearing, RunClearingRequest, RunClearingResponse};
use std::collections::BTreeSet;

pub type RawCipherText = HexBinary;

#[derive(Clone, Debug)]
pub struct MtcsService<A> {
    sk: Arc<Mutex<Option<SigningKey>>>,
    attestor: A,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunClearingMessage {
    intents: BTreeMap<RawHash, RawCipherText>,
    liquidity_sources: BTreeSet<Addr>,
}

impl<A> MtcsService<A>
where
    A: Attestor,
{
    pub fn new(sk: Arc<Mutex<Option<SigningKey>>>, attestor: A) -> Self {
        Self { sk, attestor }
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
        let liquidity_sources: Vec<Addr> = message.liquidity_sources.into_iter().collect();
        let digests_ciphertexts = message.intents;
        let (digests, ciphertexts): (Vec<_>, Vec<_>) = digests_ciphertexts.into_iter().unzip();

        let sk = self.sk.lock().unwrap();
        let obligations: Vec<SimpleObligation<Addr, i64>> = ciphertexts
            .into_iter()
            .map(|ciphertext| decrypt_obligation(sk.as_ref().unwrap(), &ciphertext))
            .collect();

        let mut mtcs = ComplexIdMtcs::wrapping(DefaultMtcs::new(PrimalDual::default()));
        let setoffs: Vec<SimpleSetoff<Addr, i64>> = mtcs.run(obligations).unwrap();

        let setoffs_enc: BTreeMap<RawHash, SettleOff> = setoffs
            .into_iter()
            .map(|so| into_settle_offs(so, &liquidity_sources))
            .zip(digests)
            .map(|(settle_off, digest)| (digest, settle_off))
            .collect();

        let msg = SubmitSetoffsMsg { setoffs_enc };

        let attestation = self
            .attestor
            .quote(msg.clone())
            .map_err(|e| Status::internal(e.to_string()))?;

        let attested_msg = RawAttested { msg, attestation };
        let message = serde_json::to_string(&attested_msg).unwrap();
        Ok(Response::new(RunClearingResponse { message }))
    }
}

fn into_settle_offs(
    so: SimpleSetoff<Addr, i64>,
    liquidity_sources: &Vec<Addr>,
) -> SettleOff {
    println!("setoff: {:?}", so);
    println!("liq sources: {:?}", liquidity_sources);

    if liquidity_sources.contains(&&so.debtor) {
        // A setoff on a tender should result in the creditor's (i.e. the tender receiver) balance
        // decreasing by the setoff amount
        SettleOff::Transfer(Transfer {
            payer: so.creditor.clone(),
            payee: so.debtor.clone(),
            amount: so.set_off as u64,
        })
    } else if liquidity_sources.contains(&&so.creditor) {
        // A setoff on an acceptance should result in the debtor's (i.e. the acceptance initiator)
        // balance increasing by the setoff amount
        SettleOff::Transfer(Transfer {
            payer: so.creditor.clone(),
            payee: so.debtor.clone(),
            amount: so.set_off as u64,
        })
    } else {
        // TODO: Add logic after implementing epoch-persistence 
        // SettleOff::SetOff(encrypt_setoff(so, debtor_pk, creditor_pk))
        
        // Need to do a no-op here
        SettleOff::Transfer(Transfer {
            payer: Addr::unchecked("0"),
            payee: Addr::unchecked("0"),
            amount: so.set_off as u64,
        })
    }
}

// fn wasm_address(pk: VerifyingKey) -> String {
//     let tm_pk = TmAccountId::from(pk);
//     AccountId::new("wasm", tm_pk.as_bytes())
//         .unwrap()
//         .to_string()
// }

// fn encrypt_setoff(
//     so: SimpleSetoff<HexBinary, i64>,
//     debtor_pk: VerifyingKey,
//     creditor_pk: VerifyingKey,
// ) -> Vec<RawCipherText> {
//     let so_ser = serde_json::to_string(&so).expect("infallible serializer");
//     let so_debtor = encrypt(&debtor_pk.to_sec1_bytes(), so_ser.as_bytes()).unwrap();
//     let so_creditor = encrypt(&creditor_pk.to_sec1_bytes(), so_ser.as_bytes()).unwrap();

//     vec![so_debtor.into(), so_creditor.into()]
// }

fn decrypt_obligation(
    sk: &SigningKey,
    ciphertext: &RawCipherText,
) -> SimpleObligation<Addr, i64> {
    let o: ContractObligation = {
        let o = decrypt(&sk.to_bytes(), ciphertext).unwrap();
        serde_json::from_slice(&o).unwrap()
    };
    SimpleObligation::new(None, o.debtor, o.creditor, i64::try_from(o.amount).unwrap()).unwrap()
}
