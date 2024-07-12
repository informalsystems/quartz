use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg /*QueryMsg, TcbCertificate*/};
use crate::state::{TcbInfo, DATABASE, ROOT_CERTIFICATE};
use der::{DecodePem};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use mc_attestation_verifier::{SignedTcbInfo};
use p256::ecdsa::VerifyingKey;
use der::DateTime;
use core::time::Duration;
use x509_cert::Certificate;
use quartz_tee_ra::intel_sgx::dcap::certificate_chain::TlsCertificateChainVerifier;
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
    let  _root = Certificate::from_pem(msg.root).expect("could not parse PEM");
    let _verifier = TlsCertificateChainVerifier::new(msg.root);
    // verifier.verify_certificate_chain(vec![root]).map_err(|_| ContractError::CertificateVerificationError)?;
   ROOT_CERTIFICATE.save(deps.storage, &msg.root.to_string()).map_err(ContractError::Std)?;
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
    let signed_tcb_info: SignedTcbInfo = SignedTcbInfo::try_from(msg.tcb_info).expect("failed to parse TCBInfo");
    let raw_root = ROOT_CERTIFICATE.load(deps.storage).unwrap();
    let  _root = Certificate::from_pem(raw_root.clone()).expect("could not parse PEM");


    let _verifier = TlsCertificateChainVerifier::new(&raw_root);
    //  verifier.verify_certificate_chain(vec![certificate, root]).map_err(|_| ContractError::CertificateVerificationError)?;
    
    // TODO: check msg.fmspc == tcb_info.fmspc

    let certificate = Certificate::from_pem(msg.certificate).expect("failed to parse PEM");
    let key = VerifyingKey::from_sec1_bytes(
        certificate
            .tbs_certificate
            .subject_public_key_info
            .subject_public_key
            .as_bytes()
            .expect("Failed to parse public key"),
    )
        .expect("Failed to decode public key");
    let time = DateTime::from_unix_duration(Duration::from_secs(msg.time)).map_err(|_| ContractError::DateTimeReadError)?;
    
    signed_tcb_info.verify(Some(&key), Some(time)).map_err(|_| ContractError::TcbInfoVerificationError)?;
    
    
    let _ = DATABASE.save(deps.storage, msg.fmspc, &TcbInfo{ info: msg.tcb_info.to_string(), certificate: msg.certificate.to_string() }).map_err(ContractError::Std);

    Ok(Response::default())
}




#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::coins;
    const TCB_SIGNER : &str = include_str!("../data/tcb_signer.pem");
    const ROOT_CA : &str = include_str!("../data/root_ca.pem");
    const TCB_INFO: &str = include_str!("../data/tcbinfo.json");
    
    #[test]
    fn verify_init_and_exec () {
        let time: u64 = 1720696777; //  Thursday, 11 July 2024 13:19:37 GMT+02:00 DST
        let info = mock_info("creator", &coins(1000, "earth"));
        let init_msg = InstantiateMsg {root: ROOT_CA };
         let mut deps = mock_dependencies();
        let res = instantiate(deps.as_mut(), mock_env(), info, init_msg);
        assert!(res.is_ok());
        
        let exec_msg = ExecuteMsg{
            fmspc: hex::decode("00606a000000").unwrap().try_into().unwrap(),
            tcb_info: TCB_INFO,
            certificate: TCB_SIGNER,
            time
        };
        let info = mock_info("creator", &coins(1000, "earth"));
        let exec = execute(deps.as_mut(), mock_env(), info, exec_msg);
        assert!(exec.is_ok());
    }
}
 
