use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use cosmwasm_std::HexBinary;
use cw_tee_mtcs::{
    msg::execute::SubmitSetoffsMsg,
    state::{RawCipherText, RawHash},
};
use ecies::{decrypt, encrypt};
use k256::ecdsa::{SigningKey, VerifyingKey};
use mtcs::{
    algo::mcmf::primal_dual::PrimalDual, impls::complex_id::ComplexIdMtcs,
    obligation::SimpleObligation, prelude::DefaultMtcs, setoff::SimpleSetoff, Mtcs,
};
use tonic::{Request, Response, Result as TonicResult, Status};

use crate::{
    attestor::Attestor,
    proto::{clearing_server::Clearing, RunClearingRequest, RunClearingResponse},
};

#[derive(Clone, Debug)]
pub struct MtcsService<A> {
    sk: Arc<Mutex<Option<SigningKey>>>,
    _attestor: A,
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
        let message = request.into_inner().message;
        let obligations_enc: BTreeMap<RawHash, RawCipherText> =
            serde_json::from_str(&message).map_err(|e| Status::invalid_argument(e.to_string()))?;

        let sk = self.sk.lock().unwrap();
        let obligations: Vec<_> = obligations_enc
            .into_values()
            .map(|ciphertext| {
                let o = decrypt(&sk.as_ref().unwrap().to_bytes(), &ciphertext).unwrap();
                serde_json::from_slice::<SimpleObligation<HexBinary, i64>>(&o).unwrap()
            })
            .collect();

        let mut mtcs = ComplexIdMtcs::wrapping(DefaultMtcs::new(PrimalDual::default()));
        let setoffs: Vec<SimpleSetoff<HexBinary, i64>> = mtcs.run(obligations).unwrap();

        let setoffs_enc: Vec<HexBinary> = setoffs
            .into_iter()
            .flat_map(|so| {
                let debtor_pk = VerifyingKey::from_sec1_bytes(&so.debtor).unwrap();
                let creditor_pk = VerifyingKey::from_sec1_bytes(&so.creditor).unwrap();

                let so_ser = serde_json::to_string(&so).expect("infallible serializer");
                let so_debtor = encrypt(&debtor_pk.to_sec1_bytes(), so_ser.as_bytes()).unwrap();
                let so_creditor = encrypt(&creditor_pk.to_sec1_bytes(), so_ser.as_bytes()).unwrap();

                [so_debtor, so_creditor]
            })
            .map(Into::into)
            .collect();

        let message = serde_json::to_string(&SubmitSetoffsMsg { setoffs_enc }).unwrap();

        Ok(Response::new(RunClearingResponse { message }))
    }
}
