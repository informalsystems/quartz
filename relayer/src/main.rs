mod cli;

use std::{error::Error, fs::File, io::Read};

use crate::cli::Cli;
use clap::Parser;
use cosmos_sdk_proto::{
    cosmos::{
        auth::v1beta1::{
            query_client::QueryClient as AuthQueryClient, BaseAccount as RawBaseAccount,
            QueryAccountRequest,
        },
        tx::v1beta1::{service_client::ServiceClient, BroadcastMode, BroadcastTxRequest},
    },
    traits::Message,
    Any,
};
use cosmrs::{
    auth::BaseAccount,
    cosmwasm::MsgExecuteContract,
    crypto::secp256k1::{SigningKey, VerifyingKey},
    tendermint::{account::Id as TmAccountId, chain::Id as TmChainId},
    tx,
    tx::{Fee, Msg, SignDoc, SignerInfo},
    AccountId, Coin,
};
use ecies::{PublicKey, SecretKey};
use quartz_cw::msg::{
    execute::attested::Attested,
    instantiate::{CoreInstantiate, RawInstantiate},
    InstantiateMsg,
};
use quartz_proto::quartz::{core_client::CoreClient, InstantiateRequest};
use quartz_relayer::types::InstantiateResponse;
use quartz_tee_ra::intel_sgx::dcap::Collateral;
use reqwest::Client;
use subtle_encoding::base64;
use tendermint::public_key::Secp256k1 as TmPublicKey;

async fn fetch_collateral(client: &Client, url: &str) -> Result<Collateral, Box<dyn Error>> {
    let response = client.get(url).send().await?;
    let collateral = response.json::<Collateral>().await?;
    Ok(collateral)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let mut client = CoreClient::connect(args.enclave_addr.uri().to_string()).await?;
    let response = client.instantiate(InstantiateRequest {}).await?;
    let response: InstantiateResponse = response.into_inner().try_into()?;

    #[cfg(feature = "mock-sgx")]
    let attestation = {
        use quartz_cw::msg::execute::attested::MockAttestation;

        MockAttestation::default()
    };

    #[cfg(not(feature = "mock-sgx"))]
    let attestation = {
        use quartz_cw::msg::execute::attested::DcapAttestation;
        use quartz_tee_ra::intel_sgx::dcap::{Collateral, Quote3};

        let quote = gramine_sgx_ias_report(response.quote())?;
        let quote: Quote3<Vec<u8>> = serde_json::from_value(quote)?;

        // Fetch collateral data
        let http_client = Client::new();
        let url = "https://api.trustedservices.intel.com/sgx/certification/v3/pckcert";
        let collateral: Collateral = fetch_collateral(&http_client, url).await?;

        println!("TCB Info: {}", collateral.tcb_info());
        DcapAttestation::new(quote, collateral)
    };

    let cw_instantiate_msg: Attested<CoreInstantiate, _> = Attested::new(
        CoreInstantiate::new(response.into_message().into_tuple().0),
        attestation,
    );

    #[cfg(feature = "mock-sgx")]
    let raw_instantiate_msg = {
        use quartz_cw::msg::execute::attested::RawMockAttestation;

        let raw_instantiate: RawInstantiate<RawMockAttestation> =
            InstantiateMsg(cw_instantiate_msg).into();
        serde_json::to_string(&raw_instantiate)?.into_bytes()
    };

    #[cfg(not(feature = "mock-sgx"))]
    let raw_instantiate_msg = {
        use quartz_cw::msg::execute::attested::RawDcapAttestation;

        let raw_instantiate: RawInstantiate<RawDcapAttestation> =
            InstantiateMsg(cw_instantiate_msg).into();
        serde_json::to_string(&raw_instantiate)?.into_bytes()
    };

    // Read the TSP secret
    let secret = {
        let mut secret = Vec::new();
        let mut tsp_sk_file = File::open(args.secret)?;
        tsp_sk_file.read_to_end(secret.as_mut())?;
        let secret = base64::decode(secret).unwrap();
        SecretKey::parse_slice(&secret).unwrap()
    };
    let tm_pubkey = {
        let pubkey = PublicKey::from_secret_key(&secret);
        TmPublicKey::from_sec1_bytes(&pubkey.serialize()).unwrap()
    };
    let sender = {
        let tm_key = TmAccountId::from(tm_pubkey);
        AccountId::new("wasm", tm_key.as_bytes()).unwrap()
    };

    let msgs = vec![MsgExecuteContract {
        sender: sender.clone(),
        contract: args.contract.clone(),
        msg: raw_instantiate_msg,
        funds: vec![],
    }
    .to_any()
    .unwrap()];

    let account = account_info(args.node_addr.uri().clone(), sender.clone()).await?;
    let amount = Coin {
        amount: 0u128,
        denom: "cosm".parse()?,
    };
    let tx_bytes = tx_bytes(
        &secret,
        amount,
        args.gas_limit,
        tm_pubkey,
        msgs,
        account.sequence,
        account.account_number,
        &args.chain_id,
    )?;

    send_tx(args.node_addr.uri().clone(), tx_bytes).await?;

    Ok(())
}

pub async fn account_info(
    node: impl ToString,
    address: impl ToString,
) -> Result<BaseAccount, Box<dyn Error>> {
    let mut client = AuthQueryClient::connect(node.to_string()).await?;
    let request = tonic::Request::new(QueryAccountRequest {
        address: address.to_string(),
    });
    let response = client.account(request).await?;
    let response = RawBaseAccount::decode(response.into_inner().account.unwrap().value.as_slice())?;
    let account = BaseAccount::try_from(response)?;
    Ok(account)
}

#[allow(clippy::too_many_arguments)]
pub fn tx_bytes(
    secret: &SecretKey,
    amount: Coin,
    gas: u64,
    tm_pubkey: VerifyingKey,
    msgs: Vec<Any>,
    sequence_number: u64,
    account_number: u64,
    chain_id: &TmChainId,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let tx_body = tx::Body::new(msgs, "", 0u16);
    let signer_info = SignerInfo::single_direct(Some(tm_pubkey.into()), sequence_number);
    let auth_info = signer_info.auth_info(Fee::from_amount_and_gas(amount, gas));
    let sign_doc = SignDoc::new(&tx_body, &auth_info, chain_id, account_number)?;
    let tx_signed = sign_doc.sign(&SigningKey::from_slice(&secret.serialize()).unwrap())?;
    Ok(tx_signed.to_bytes()?)
}

pub async fn send_tx(node: impl ToString, tx_bytes: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let mut client = ServiceClient::connect(node.to_string()).await?;
    let request = tonic::Request::new(BroadcastTxRequest {
        tx_bytes,
        mode: BroadcastMode::Block.into(),
    });
    let _response = client.broadcast_tx(request).await?;
    Ok(())
}

#[cfg(not(feature = "mock-sgx"))]
fn gramine_sgx_ias_report(quote: &[u8]) -> Result<serde_json::Value, Box<dyn Error>> {
    use std::{fs::read_to_string, io::Write, process::Command};

    let dir = tempfile::tempdir()?;
    let quote_file_path = dir.path().join("test.quote");
    let datareport_file_path = dir.path().join("datareport");
    let datareportsig_file_path = dir.path().join("datareportsig");

    let mut quote_file = File::create(quote_file_path.clone())?;
    quote_file.write_all(quote)?;

    let gramine_sgx_ias_request_output = Command::new("gramine-sgx-ias-request")
        .arg("report")
        .args(["-g", "51CAF5A48B450D624AEFE3286D314894"])
        .args(["-k", "669244b3e6364b5888289a11d2a1726d"])
        .args(["-q", &quote_file_path.display().to_string()])
        .args(["-r", &datareport_file_path.display().to_string()])
        .args(["-s", &datareportsig_file_path.display().to_string()])
        .output()?;
    println!("{gramine_sgx_ias_request_output:?}");

    let report = read_to_string(datareport_file_path)?;
    let report_sig = read_to_string(datareportsig_file_path)?;
    let ias_report = serde_json::json!({"report": report, "reportsig": report_sig});
    Ok(ias_report)
}
