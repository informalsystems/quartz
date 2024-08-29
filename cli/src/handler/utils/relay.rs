use cosmwasm_std::Binary;
use hex::decode;
use quartz_common::{
    contract::msg::{
        execute::{
            attested::{EpidAttestation, RawAttested, RawEpidAttestation, RawMockAttestation},
            session_create::RawSessionCreate,
            session_set_pub_key::RawSessionSetPubKey,
        },
        instantiate::RawCoreInstantiate,
        RawExecuteMsg,
    },
    proto::{
        core_client::CoreClient, InstantiateRequest, SessionCreateRequest, SessionSetPubKeyRequest,
    },
};
use quartz_tee_ra::{intel_sgx::epid::types::ReportBody, IASReport};
use serde_json::json;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    process::Command,
};

use crate::error::Error;

#[derive(Debug)]
pub enum RelayMessage {
    Instantiate,
    SessionCreate,
    SessionSetPubKey(String),
}

impl RelayMessage {
    pub async fn run_relay(
        &self,
        enclave_rpc: String,
        mock_sgx: bool,
    ) -> Result<serde_json::Value, Error> {
        // Query the gRPC quartz enclave service
        let mut qc_client = CoreClient::connect(enclave_rpc)
            .await
            .map_err(|e| Error::GenericErr(e.to_string()))?;

        let attested_msg = match self {
            RelayMessage::Instantiate => &qc_client
                .instantiate(tonic::Request::new(InstantiateRequest {}))
                .await
                .map_err(|e| Error::GenericErr(e.to_string()))?
                .get_ref()
                .message
                .clone(),
            RelayMessage::SessionCreate => &qc_client
                .session_create(tonic::Request::new(SessionCreateRequest {}))
                .await
                .map_err(|e| Error::GenericErr(e.to_string()))?
                .get_ref()
                .message
                .clone(),
            RelayMessage::SessionSetPubKey(proof) => &qc_client
                .session_set_pub_key(SessionSetPubKeyRequest {
                    message: proof.to_string(),
                })
                .await
                .map_err(|e| Error::GenericErr(e.to_string()))?
                .get_ref()
                .message
                .clone(),
        };

        let mut msg_json: serde_json::Value = serde_json::from_str(attested_msg)?;
        let quote = msg_json["quote"].take();

        if mock_sgx {
            let attestation: RawMockAttestation = serde_json::from_value(quote)?;

            self.create_attested_msg(msg_json, attestation)
        } else {
            let attestation: RawEpidAttestation = create_epid_attestation(&quote).await?.into();

            self.create_attested_msg(msg_json, attestation)
        }
    }

    fn create_attested_msg<RA: serde::Serialize>(&self, msg_json: serde_json::Value, attestation: RA) -> Result<serde_json::Value, Error> {
        match self {
            RelayMessage::Instantiate => {
                let msg: RawCoreInstantiate = serde_json::from_value(msg_json)?;
                let query_result: RawAttested<RawCoreInstantiate, RA> = RawAttested {
                    msg,
                    attestation,
                };
                Ok(json!(query_result))
            },
            RelayMessage::SessionCreate => {
                let msg: RawSessionCreate = serde_json::from_value(msg_json)?;
                let query_result: RawExecuteMsg<RA> = RawExecuteMsg::RawSessionCreate(RawAttested {
                    msg,
                    attestation,
                });
                Ok(json!({ "quartz": query_result }))
            },
            RelayMessage::SessionSetPubKey(_) => {
                let msg: RawSessionSetPubKey = serde_json::from_value(msg_json)?;
                let query_result: RawExecuteMsg<RA> = RawExecuteMsg::RawSessionSetPubKey(RawAttested {
                    msg,
                    attestation,
                });
                Ok(json!({ "quartz": query_result }))
            }
        }
    }
}

async fn create_epid_attestation(quote: &serde_json::Value) -> Result<EpidAttestation, Error> {
    let quote_str = quote
        .as_str()
        .ok_or_else(|| Error::GenericErr("quote is not a string".to_string()))?;
    let quote = decode(quote_str).map_err(|e| Error::GenericErr(e.to_string()))?;

    let (report, report_sig) = run_docker_command(&quote).await?;

    let report_json: ReportBody = serde_json::from_str(&report)?;
    let report_sig = report_sig.replace('\n', "");

    let ias_report = IASReport {
        report: report_json,
        report_sig: Binary::from_base64(&report_sig)
            .map_err(|e| Error::GenericErr(e.to_string()))?,
    };

    Ok(EpidAttestation::new(ias_report))
}

async fn run_docker_command(quote: &[u8]) -> Result<(String, String), Error> {
    let dir = tempfile::tempdir()?;
    let ias_api_key: &str = "669244b3e6364b5888289a11d2a1726d";
    let ra_client_spid: &str = "51CAF5A48B450D624AEFE3286D314894";
    let quote_file_path = dir.path().join("test.quote");
    let datareport_file_path = dir.path().join("datareport");
    let datareportsig_file_path = dir.path().join("datareportsig");

    let mut quote_file = File::create(quote_file_path.clone()).await?;
    quote_file.write_all(quote).await?;

    let status = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-it")
        .arg("-v")
        .arg("/tmp:/tmp:rw")
        .arg("gramineproject/gramine:1.7-jammy")
        .arg(format!(
            "gramine-sgx-ias-request report -g \"{}\" -k \"{}\" -q \"{}\" -r \"{}\" -s \"{}\" > /dev/null 2>&1",
            ra_client_spid, ias_api_key, quote_file_path.display(), datareport_file_path.display(), datareportsig_file_path.display()
        ))
        .status()
        .await
        .map_err(|e| Error::GenericErr(e.to_string()))?;

    if !status.success() {
        return Err(Error::GenericErr(
            "Failed to run docker command".to_string(),
        ));
    }

    let report = fs::read_to_string(datareport_file_path)
        .await
        .map_err(|e| Error::GenericErr(e.to_string()))?;

    let reportsig = fs::read_to_string(datareportsig_file_path)
        .await
        .map_err(|e| Error::GenericErr(e.to_string()))?
        .replace('\r', "");

    Ok((report, reportsig))
}
