use std::str::FromStr;

use commit_reveal_contract::msg::{execute::Ping, ExecuteMsg};
use cosmrs::{tendermint::chain::Id as ChainId, AccountId};
use cosmwasm_std::HexBinary;
use cw_client::{CliClient, CwClient};
use ecies::{decrypt, encrypt};
use hex;
use k256::ecdsa::{SigningKey, VerifyingKey};
use rand_core::OsRng;
use reqwest::Url;
use serde_json::json;

#[tokio::main]
async fn main() {
    // Enclave public key
    let decoded: Vec<u8> =
        hex::decode("031cfdb9bc7eb0c75a715e2d609b7407dcaebc531fb8c51c6168787b480097888b")
            .expect("Decoding failed");
    let pk_hex = VerifyingKey::from_sec1_bytes(&decoded).unwrap();

    // Encrypt message to public key
    let plaintext_msg = "hello";
    let serialized_msg = serde_json::to_string(plaintext_msg).expect("infallible serializer");
    let encrypted_msg: HexBinary = encrypt(&pk_hex.to_sec1_bytes(), serialized_msg.as_bytes())
        .unwrap()
        .into();

    // Prepare cosmwasm message
    // Set pubkey to user's pubkey
    let reveal_msg: ExecuteMsg = ExecuteMsg::Ping(Ping {
        pubkey: HexBinary::from_hex(
            "026452a47ff13ef0aefcaf79b4e68389c55759abaa644166e99c1b9bfc904597d4",
        )
        .unwrap(),
        message: encrypted_msg,
    });

    // Send transaction to chain
    let cw_client = CliClient::neutrond(Url::from_str("http://127.0.0.1:26657").unwrap());

    let chain_id = &ChainId::from_str("test-1").unwrap();
    let output = cw_client
        .tx_execute(
            &AccountId::from_str(
                "neutron1u0ehv853npcmu9m4jexampykq6yeuf6nlnxpvm5m8w73g2vrv9wqyx0mdp",
            )
            .unwrap(),
            chain_id,
            2000000,
            "admin",
            json!(reveal_msg),
            "11000untrn",
        )
        .await;

    println!("Output TX: {:?}", output);
}

// For decrypting response from enclave
#[test]
fn decrypt_2() {
    let decoded: Vec<u8> =
        hex::decode("3255344d1af484a7aacce85a49897c69099bceb99cba9126cc1229a18926d3c3")
            .expect("Decoding failed");

    let res = decrypt(&decoded, &hex::decode("04051cabe31a3ec2721e06a7fa0acad190b62e1d66ecf67136fd6dba0f0c13c4ca3fada53a3bcf95ee7e150f0a94457dc90a5efc4e28550440dd3cad4e5519e9b6897e9f43757068e40463236c90dbc61d4c1048b0d6e3b588e822fe2b6f43e2d4bd6fc30a08603bb05f1ae0f3783f09b4bcad2f000967df61aca620").unwrap()).unwrap();
    println!("{}", String::from_utf8(res).unwrap());
}

// Generate pubkey pair for user
#[test]
fn gen() {
    // Generate a random secret key
    let signing_key = SigningKey::random(&mut OsRng);

    // Get the verifying (public) key
    let verifying_key = signing_key.verifying_key();

    // Serialize the public key in compressed SEC1 format
    let pk_hex = hex::encode(verifying_key.to_encoded_point(true).as_bytes());

    // Serialize the secret key
    let sk_bytes = signing_key.to_bytes();
    let sk_hex = hex::encode(sk_bytes);

    // Print the keys
    println!("Secret Key: {}", sk_hex);
    println!("Public Key (SEC1 compressed): {}", pk_hex);
}
