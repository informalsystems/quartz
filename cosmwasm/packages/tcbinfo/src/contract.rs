use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetTcbInfoResponse, InstantiateMsg, QueryMsg};
use crate::state::{TcbInfo, DATABASE, ROOT_CERTIFICATE};
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
    let root = Certificate::from_pem(&msg.root).expect("could not parse PEM");
    let verifier = TlsCertificateChainVerifier::new(&msg.root);
     verifier.verify_certificate_chain(vec![&root], vec![], None).map_err(|_| ContractError::CertificateVerificationError)?;
    ROOT_CERTIFICATE
        .save(deps.storage, &msg.root.to_string())
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
     let tcbinfo_raw: Value = serde_json::from_str(&msg.tcb_info).map_err(|_| ContractError::TcbInfoReadError)?;
    let fmspc_raw = hex::decode(tcbinfo_raw.get("tcbInfo").unwrap().get("fmspc").unwrap().as_str().expect("could not find fmspc string")).map_err(|_| ContractError::TcbInfoReadError)?;
     let fmspc: [u8; 6] = fmspc_raw.try_into().unwrap();

    let certificate = Certificate::from_pem(msg.certificate.clone()).expect("failed to parse PEM");
    verifier.verify_certificate_chain(vec![&certificate, &root], vec![], None).map_err(|_| ContractError::CertificateVerificationError)?;
    let key = VerifyingKey::from_sec1_bytes(
        certificate
            .tbs_certificate
            .subject_public_key_info
            .subject_public_key
            .as_bytes()
            .expect("Failed to parse public key"),
    )
        .expect("Failed to decode public key");
    let time = msg
        .time
        .parse::<DateTime>()
        .map_err(|_| ContractError::DateTimeReadError)?;
    signed_tcb_info
        .verify(Some(&key), Some(time))
        .map_err(|_| ContractError::TcbInfoVerificationError)?;

    let _ = DATABASE
        .save(
            deps.storage,
            fmspc,
            &TcbInfo {
                info: msg.tcb_info.to_string(),
                certificate: msg.certificate.to_string(),
            },
        )
        .map_err(ContractError::Std);

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetTcbInfo { fmspc, time } => {
            to_json_binary(&query::get_info(deps, fmspc, time)?)
        }
    }
}

pub mod query {
    use super::*;

    pub fn get_info(deps: Deps, fmspc: [u8; 6], time: String) -> StdResult<GetTcbInfoResponse> {
        let tcb_info = DATABASE.load(deps.storage, fmspc)?;
        verify_tcb_info(&tcb_info, time)?;
        Ok(GetTcbInfoResponse {
            tcb_info: tcb_info.info,
        })
    }

    fn verify_tcb_info<'a>(tcb_info: &'a TcbInfo, time: String) -> StdResult<()> {
        let signed_tcb_info: SignedTcbInfo =
            SignedTcbInfo::try_from(tcb_info.info.as_ref()).unwrap();
        let certificate = Certificate::from_pem(&tcb_info.certificate).unwrap();
        let key = VerifyingKey::from_sec1_bytes(
            certificate
                .tbs_certificate
                .subject_public_key_info
                .subject_public_key
                .as_bytes()
                .expect("Failed to parse public key"),
        )
        .expect("Failed to decode public key");
        let time = time
            .parse::<DateTime>()
            .map_err(|_| StdError::generic_err("Invalid timestamp"))?;
        signed_tcb_info
            .verify(Some(&key), Some(time))
            .map_err(|_| StdError::generic_err("TCBInfo verification failed"))
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        coins,
        testing::{mock_dependencies, mock_env, mock_info},
    };
    use super::*;
    const TCB_SIGNER: &str = include_str!("../data/tcb_signer.pem");
    const ROOT_CA: &str = include_str!("../data/root_ca.pem");
    const TCB_INFO: &str = include_str!("../data/tcbinfo.json");
    const FMSPC: &str = "00606a000000";
    const TIME: &str = "2024-07-15T15:19:13Z";
    #[test]
    fn verify_init_and_exec() {
        let time = "2024-07-11T15:19:13Z";
        let info = mock_info("creator", &coins(1000, "earth"));
        let init_msg = InstantiateMsg { root: ROOT_CA.to_string() };
        let mut deps = mock_dependencies();
        let res = instantiate(deps.as_mut(), mock_env(), info, init_msg);
        assert!(res.is_ok());

        let exec_msg = ExecuteMsg {
            tcb_info: TCB_INFO.to_string(),
            certificate: TCB_SIGNER.to_string(),
            time: time.to_string(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));
        let exec = execute(deps.as_mut(), mock_env(), info, exec_msg);
        assert!(exec.is_ok());
        let query = query(deps.as_ref(), mock_env(),  QueryMsg::GetTcbInfo { fmspc: hex::decode(FMSPC).unwrap().try_into().unwrap(), time: TIME.to_string()});
    assert!(query.is_ok());
        println!("{:?}", query.unwrap());
    }
}
