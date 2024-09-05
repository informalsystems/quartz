#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;
use der::{DateTime, DecodePem};
use mc_attestation_verifier::{CertificateChainVerifier, SignedTcbInfo};
use p256::ecdsa::VerifyingKey;
use quartz_tee_ra::intel_sgx::dcap::certificate_chain::TlsCertificateChainVerifier;
use serde_json::Value;
use x509_cert::Certificate;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, GetTcbInfoResponse, InstantiateMsg, QueryMsg},
    state::{TcbInfo, DATABASE, ROOT_CERTIFICATE},
};
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:tcbinfo";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let root = Certificate::from_pem(&msg.root_cert).expect("could not parse PEM");
    let verifier = TlsCertificateChainVerifier::new(&msg.root_cert);
    verifier
        .verify_certificate_chain(vec![&root], vec![], None)
        .map_err(|_| ContractError::CertificateVerificationError)?;
    ROOT_CERTIFICATE
        .save(deps.storage, &msg.root_cert.to_string())
        .map_err(ContractError::Std)?;
    assert!(DATABASE.is_empty(deps.storage));
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let signed_tcb_info: SignedTcbInfo =
        SignedTcbInfo::try_from(msg.tcb_info.as_ref()).expect("failed to parse TCBInfo");
    let raw_root = ROOT_CERTIFICATE.load(deps.storage).unwrap();
    let root = Certificate::from_pem(raw_root.clone()).expect("could not parse PEM");
    let verifier = TlsCertificateChainVerifier::new(&raw_root);
    let fmspc = execute::get_fmspc(&msg.tcb_info);
    let certificate = Certificate::from_pem(msg.certificate.clone()).expect("failed to parse PEM");

    let time = msg
        .time
        .map(|time| time.parse::<DateTime>().expect("could not parse datetime"));

    assert!(
        execute::check_certificate_validity(&root, time),
        "On-chain root certificate validity check failed"
    );
    assert!(
        execute::check_certificate_validity(&certificate, time),
        "Certificate validity check failed"
    );

    let key = VerifyingKey::from_sec1_bytes(
        certificate
            .tbs_certificate
            .subject_public_key_info
            .subject_public_key
            .as_bytes()
            .expect("Failed to parse public key"),
    )
    .expect("Failed to decode public key");

    verifier
        .verify_certificate_chain(vec![&certificate, &root], vec![], None)
        .map_err(|_| ContractError::CertificateVerificationError)?;

    signed_tcb_info
        .verify(Some(&key), time)
        .map_err(|_| ContractError::TcbInfoVerificationError)?;

    let _ = DATABASE
        .save(
            deps.storage,
            fmspc,
            &TcbInfo {
                info: msg.tcb_info.to_string(),
                //  certificate: msg.certificate.to_string(),
            },
        )
        .map_err(ContractError::Std);

    Ok(Response::default())
}

pub mod execute {
    use super::*;

    pub fn get_fmspc(tcbinfo: &str) -> [u8; 6] {
        let tcbinfo_raw: Value = serde_json::from_str(tcbinfo).expect("could not read tcbinfo");
        let fmspc_raw = hex::decode(
            tcbinfo_raw
                .get("tcbInfo")
                .unwrap()
                .get("fmspc")
                .unwrap()
                .as_str()
                .expect("could not find fmspc string"),
        )
        .expect("failed to decode fmspc hex string");
        fmspc_raw.try_into().unwrap()
    }

    pub fn check_certificate_validity(cert: &Certificate, time: Option<DateTime>) -> bool {
        match time {
            None => true,
            Some(time) => {
                let validity = cert.tbs_certificate.validity;
                let start = validity.not_before.to_date_time();
                let end = validity.not_after.to_date_time();
                time >= start && time <= end
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetTcbInfo { fmspc } => to_json_binary(&query::get_info(deps, fmspc)?),
    }
}

pub mod query {
    use super::*;

    pub fn get_info(deps: Deps, fmspc: String) -> StdResult<GetTcbInfoResponse> {
        let key: [u8; 6] = hex::decode(fmspc)
            .unwrap()
            .try_into()
            .expect("invalid fmspc");
        let tcb_info = DATABASE.load(deps.storage, key)?;
        let tcb_info_response = serde_json::from_str(&tcb_info.info).map_err(|_| {
            StdError::parse_err(tcb_info.info, "Could not prarse on-chain TcbInfo as JSON")
        })?;
        Ok(GetTcbInfoResponse {
            tcb_info: tcb_info_response,
        })
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        coins,
        testing::{message_info, mock_dependencies, mock_env},
    };

    use super::*;
    const TCB_SIGNER: &str = include_str!("../data/tcb_signer.pem");
    const ROOT_CA: &str = include_str!("../data/root_ca.pem");
    const TCB_INFO: &str = include_str!("../data/tcbinfo.json");
    const FMSPC: &str = "00606a000000";
    // const TIME: &str = "2024-07-15T15:19:13Z";
    #[test]
    fn verify_init_and_exec() {
        let time = "2024-07-11T15:19:13Z";
        let deps = mock_dependencies();
        let creator = deps.api.addr_make("creator");

        let info = message_info(&creator, &coins(1000, "earth"));
        let init_msg = InstantiateMsg {
            root_cert: ROOT_CA.to_string(),
        };
        let mut deps = mock_dependencies();
        let res = instantiate(deps.as_mut(), mock_env(), info, init_msg);
        assert!(res.is_ok());

        let exec_msg = ExecuteMsg {
            tcb_info: TCB_INFO.to_string(),
            certificate: TCB_SIGNER.to_string(),
            time: Some(time.to_string()),
        };
        let info = message_info(&creator, &coins(1000, "earth"));
        let exec = execute(deps.as_mut(), mock_env(), info, exec_msg);
        assert!(exec.is_ok());
        let query = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetTcbInfo {
                fmspc: FMSPC.to_string(),
            },
        );
        assert!(query.is_ok());
        println!("{:?}", query.unwrap());
    }
}
