use std::str::FromStr;

use cosmrs::{tendermint::chain::Id as ChainId, AccountId};
use cosmwasm_std::HexBinary;
use cw_client::{CliClient, CwClient};
use ecies::encrypt;
use k256::ecdsa::VerifyingKey;
use ping_pong_contract::msg::{execute::Ping, ExecuteMsg};
use reqwest::Url;
use serde_json::json;

#[tokio::main]
async fn main() {
    // Enclave public key
    let decoded: Vec<u8> =
        hex::decode("02a353f7bfdab237a12628724e4f33935d8300bd5dbcdb2115105e46de40bfd333")
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
    let pong_msg: ExecuteMsg = ExecuteMsg::Ping(Ping {
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
                "neutron1des3nftm0wd7qhxc3dwlflk46rjucg5rlrcugukdpv7r4ly2ru8scx2su4",
            )
            .unwrap(),
            chain_id,
            2000000,
            "admin",
            json!(pong_msg),
            "11000untrn",
        )
        .await;

    println!("Output TX: {:?}", output);
}

// For decrypting response from enclave
#[test]
fn decrypt_enclave_response() {
    let decoded: Vec<u8> =
        hex::decode("3255344d1af484a7aacce85a49897c69099bceb99cba9126cc1229a18926d3c3")
            .expect("Decoding failed");

    let res = decrypt(&decoded, &hex::decode("043647ded7ab2cc5f2f93a6d026af8481c66cc717b0aff98e933679774ecedcd6d6df27c2e8b1e3155edc8af4d87137d2f379d71001aa4d91ee4d37f037a3d16c88afc5fc351b4fd0201e9969ea3a5deb556174f13d91fac2f490d242852b697a9ca55e9e32f46df3fe902ce17514ad4e6d48fa3b15c4c747fb491c938").unwrap()).unwrap();
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
