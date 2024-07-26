use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};

use cosmrs::{tendermint::account::Id as TmAccountId, AccountId};
use cosmwasm_std::{Addr, HexBinary, Uint128};
//TODO: get rid of this
use cw_tee_mtcs::{
    msg::execute::SubmitSetoffsMsg,
    state::{LiquiditySource, LiquiditySourceType, RawHash, SettleOff, Transfer},
};
use ecies::{decrypt, encrypt};
use k256::ecdsa::{SigningKey, VerifyingKey};
use mtcs::{
    algo::mcmf::primal_dual::PrimalDual, impls::complex_id::ComplexIdMtcs,
    obligation::SimpleObligation, prelude::DefaultMtcs, setoff::SimpleSetoff, Mtcs,
};
use quartz_common::{
    contract::{msg::execute::attested::RawAttested, state::Config},
    enclave::{attestor::Attestor, server::ProofOfPublication},
};
use serde::{Deserialize, Serialize};
use tonic::{Request, Response, Result as TonicResult, Status};

use crate::{
    proto::{clearing_server::Clearing, RunClearingRequest, RunClearingResponse},
    types::ContractObligation,
};

pub type RawCipherText = HexBinary;

#[derive(Clone, Debug)]
pub struct MtcsService<A> {
    config: Config,
    sk: Arc<Mutex<Option<SigningKey>>>,
    attestor: A,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunClearingMessage {
    intents: BTreeMap<RawHash, RawCipherText>,
    liquidity_sources: BTreeSet<LiquiditySource>,
}

impl<A> MtcsService<A>
where
    A: Attestor,
{
    pub fn new(config: Config, sk: Arc<Mutex<Option<SigningKey>>>, attestor: A) -> Self {
        Self {
            config,
            sk,
            attestor,
        }
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
        // TODO: ensure no duplicates somewhere else!
        let liquidity_sources: Vec<LiquiditySource> =
            message.liquidity_sources.into_iter().collect();
        let digests_ciphertexts: BTreeMap<HexBinary, HexBinary> = message.intents;
        let (digests, ciphertexts): (Vec<_>, Vec<_>) = digests_ciphertexts.into_iter().unzip();

        let sk = self.sk.lock().unwrap();
        let obligations: Vec<SimpleObligation<LiquiditySource, i64>> = ciphertexts
            .into_iter()
            .map(|ciphertext| decrypt_obligation(sk.as_ref().unwrap(), &ciphertext))
            .collect();

        let mut mtcs = ComplexIdMtcs::wrapping(DefaultMtcs::new(PrimalDual::default()));

        let setoffs: Vec<SimpleSetoff<LiquiditySource, i64>> = mtcs.run(obligations).unwrap();
        let setoffs_enc: BTreeMap<RawHash, SettleOff> = setoffs
            .into_iter()
            .map(|so| into_settle_offs(so, &liquidity_sources))
            .zip(digests)
            .map(|(settle_off, digest)| (digest, settle_off))
            .collect();

        let msg = SubmitSetoffsMsg { setoffs_enc };
        println!("setoff_msg: {:?}", msg);
        let attestation = self
            .attestor
            .quote(msg.clone())
            .map_err(|e| Status::internal(e.to_string()))?;

        let attested_msg = RawAttested { msg, attestation };
        let message = serde_json::to_string(&attested_msg).unwrap();
        Ok(Response::new(RunClearingResponse { message }))
    }
}

// TODO Switch from Vec<_> to Vec<LiquiditySource>
fn into_settle_offs(
    so: SimpleSetoff<LiquiditySource, i64>,
    liquidity_sources: &Vec<LiquiditySource>,
) -> SettleOff {
    println!("\nsetoff: {:?}", so);
    println!("\nliq sources: {:?}", liquidity_sources);

    // TODO: temporary patch, fix issue with liquidity sources becoming type External
    if liquidity_sources
        .iter()
        .map(|lqs| lqs.address.clone())
        .collect::<Vec<Addr>>()
        .contains(&&so.debtor.address)
    {
        // A setoff on a tender should result in the creditor's (i.e. the tender receiver) balance
        // decreasing by the setoff amount
        SettleOff::Transfer(Transfer {
            payer: so.creditor.address.clone(),
            payee: so.debtor.address.clone(),
            // TODO: Include denominations
            amount: ("peppicoin".to_owned(), Uint128::from(so.set_off as u128)),
        })
    } else if liquidity_sources
        .iter()
        .map(|lqs| lqs.address.clone())
        .collect::<Vec<Addr>>()
        .contains(&&so.creditor.address)
    {
        // A setoff on an acceptance should result in the debtor's (i.e. the acceptance initiator)
        // balance increasing by the setoff amount
        SettleOff::Transfer(Transfer {
            payer: so.creditor.address.clone(),
            payee: so.debtor.address.clone(),
            amount: ("peppicoin".to_owned(), Uint128::from(so.set_off as u128)),
        })
    } else {
        // TODO: Tracked by issue #22

        // A no-op for the time being.
        SettleOff::SetOff(vec![])
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
) -> SimpleObligation<LiquiditySource, i64> {
    let o: ContractObligation = {
        let o = decrypt(&sk.to_bytes(), ciphertext).unwrap();
        serde_json::from_slice(&o).unwrap()
    };

    SimpleObligation::new(
        None,
        LiquiditySource {
            address: o.debtor,
            source_type: LiquiditySourceType::External,
        },
        LiquiditySource {
            address: o.creditor,
            source_type: LiquiditySourceType::External,
        },
        i64::try_from(o.amount).unwrap(),
    )
    .unwrap()
}
