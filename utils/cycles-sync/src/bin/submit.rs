use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
    str::FromStr,
    env
};

use bip32::{
    secp256k1::{
        ecdsa::VerifyingKey,
        sha2::{Digest, Sha256},
    },
    Error as Bip32Error, Language, Mnemonic, Prefix, PrivateKey, Seed, XPrv,
};
use cosmrs::{tendermint::account::Id as TmAccountId, tendermint::chain::Id as TmChainId, AccountId};
use cosmwasm_std::{Addr, HexBinary, StdError};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, Level};
use uuid::Uuid;
use anyhow::anyhow;
use subtle_encoding::{bech32::decode as bech32_decode, Error as Bech32DecodeError};


use cycles_sync::{
    obligato_client::{http::HttpClient, Client},
    types::{
        Obligation, ObligatoObligation, ObligatoSetOff, ContractObligation, RawEncryptedObligation, RawObligation,
        RawOffset, RawSetOff, SubmitObligationsMsg, SubmitObligationsMsgInner,
    },
    wasmd_client::{CliWasmdClient, QueryResult, WasmdClient},
};

use reqwest::Url;

const MNEMONIC_PHRASE: &str = "clutch debate vintage foster barely primary clown leader sell manual leopard ladder wet must embody story oyster imitate cable alien six square rice wedding";

const ADDRESS_PREFIX: &str = "wasm";

type Sha256Digest = [u8; 32];

#[derive(Clone, Debug, Serialize, Deserialize)]
struct QueryAllSetoffsResponse {
    setoffs: Vec<(HexBinary, RawSetOff)>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        panic!()
    }
    
    let epoch_pk = &args[1];
    let contract: AccountId = wasmaddr_to_id(&args[2])?;
    let flip: bool = bool::from_str(&args[3])?;

    let node: Url = Url::parse("http://143.244.186.205:26657")?;
    let chain_id: TmChainId = TmChainId::from_str("testing")?;

    // TODO: replace Addr with string, probably

    let admin = Addr::unchecked("wasm14qdftsfk6fwn40l0xmruga08xlczl4g05npy70");


    let mut alice = Addr::unchecked("wasm124tuy67a9dcvfgcr4gjmz60syd8ddaugl33v0n");
    let mut bob = Addr::unchecked("wasm1ctkqmg45u85jnf5ur9796h7ze4hj6ep5y7m7l6");    

    if flip {
        let temp = alice.clone();
        alice = bob;
        bob = temp;
    }

    let overdraft = Addr::unchecked("wasm1huhuswjxfydydxvdadqqsaet2p72wshtmr72yzx09zxncxtndf2sqs24hk");

    let alice_to_bob: ContractObligation = ContractObligation {
        debtor: alice.clone(),
        creditor: bob.clone(),
        amount: 10,
        salt: HexBinary::from([0; 64]),
    };

    let bob_acceptance: ContractObligation = ContractObligation {
        debtor: bob.clone(),
        creditor: overdraft.clone(),
        amount: 10,
        salt: HexBinary::from([0; 64]),
    };

    let alice_tender: ContractObligation = ContractObligation {
        debtor: overdraft.clone(),
        creditor: alice.clone(),
        amount: 10,
        salt: HexBinary::from([0; 64]),
    };

    let intents = vec![alice_to_bob, bob_acceptance, alice_tender];
    println!("intents: {:?}", intents);

    let epoch_pk = VerifyingKey::from_sec1_bytes(&hex::decode(epoch_pk).unwrap()).unwrap();

    let intents_enc = encrypt_overdraft_intents(
                    intents, 
                    &epoch_pk);

    let liquidity_sources = vec![overdraft];

    let msg = create_wasm_msg(intents_enc, liquidity_sources)?;
    let wasmd_client = CliWasmdClient::new(node);
    wasmd_client.tx_execute(&contract, &chain_id, 3000000, admin.to_string(), msg)?;              

    Ok(())
}

pub struct OverdraftObligation {
    pub debtor: Addr,
    pub creditor: Addr,
    pub amount: u64,
}

fn encrypt_overdraft_intents(
    intents: Vec<ContractObligation>,
    epoch_pk: &VerifyingKey,
) -> Vec<(Sha256Digest, Vec<u8>)> {
    let mut intents_enc = vec![];

    for i in intents {
        // serialize intent
        let i_ser = serde_json::to_string(&i).unwrap();

        // encrypt intent
        let i_cipher = ecies::encrypt(&epoch_pk.to_sec1_bytes(), i_ser.as_bytes()).unwrap();

        // hash intent
        let i_digest: Sha256Digest = {
            let mut hasher = Sha256::new();
            hasher.update(i_ser);
            hasher.finalize().into()
        };

        intents_enc.push((i_digest, i_cipher));
    }

    intents_enc
}



fn address_from_uuid(uuid: Uuid) -> anyhow::Result<()> {
    let seed = global_seed()?;
    let sk = derive_child_xprv(&seed, uuid);
    let pk_b = sk.public_key().public_key().to_sec1_bytes();
    let pk = VerifyingKey::from_sec1_bytes(&pk_b)?;
    println!("{}", wasm_address(pk));
    Ok(())
}

fn wasm_address(pk: VerifyingKey) -> String {
    let tm_pk = TmAccountId::from(pk);
    AccountId::new(ADDRESS_PREFIX, tm_pk.as_bytes())
        .unwrap()
        .to_string()
}

fn global_seed() -> Result<Seed, Bip32Error> {
    let mnemonic = Mnemonic::new(MNEMONIC_PHRASE, Language::English)?;
    Ok(mnemonic.to_seed("password"))
}

fn create_wasm_msg(
    obligations_enc: Vec<(Sha256Digest, Vec<u8>)>,
    liquidity_sources: Vec<Addr>,
) -> anyhow::Result<serde_json::Value> {
    let obligations_enc: Vec<_> = obligations_enc
        .into_iter()
        .map(|(digest, ciphertext)| {
            let digest = HexBinary::from(digest);
            let ciphertext = HexBinary::from(ciphertext);
            RawEncryptedObligation { digest, ciphertext }
        })
        .collect();

    let liquidity_sources = liquidity_sources
        .into_iter()
        .map(|addr| HexBinary::from(addr.as_bytes()))
        .collect();

    let msg = SubmitObligationsMsg {
        submit_obligations: SubmitObligationsMsgInner {
            obligations: obligations_enc,
            liquidity_sources,
        },
    };
    serde_json::to_value(msg).map_err(Into::into)
}

fn encrypt_intents(
    intents: Vec<ObligatoObligation>,
    keys: &HashMap<Uuid, XPrv>,
    epoch_pk: &VerifyingKey,
) -> Vec<(Sha256Digest, Vec<u8>)> {
    let mut intents_enc = vec![];
    let mut intent_user_map = HashMap::new();

    for i in intents {
        // create an intent
        let ro = {
            let o = Obligation {
                debtor: keys[&i.debtor_id].private_key().public_key(),
                creditor: keys[&i.creditor_id].private_key().public_key(),
                amount: i.amount,
                salt: [0; 64],
            };
            RawObligation::from(o)
        };

        // serialize intent
        let i_ser = serde_json::to_string(&ro).unwrap();

        // encrypt intent
        let i_cipher = ecies::encrypt(&epoch_pk.to_sec1_bytes(), i_ser.as_bytes()).unwrap();

        // hash intent
        let i_digest: Sha256Digest = {
            let mut hasher = Sha256::new();
            hasher.update(i_ser);
            hasher.finalize().into()
        };

        intents_enc.push((i_digest, i_cipher));
        intent_user_map.insert(HexBinary::from(i_digest), (i.debtor_id, i.creditor_id));
    }

    intents_enc
}

fn add_default_acceptances(obligations: &mut Vec<ObligatoObligation>, liquidity_sources: &[Uuid]) {
    let acceptances = obligations.iter().fold(HashSet::new(), |mut acc, o| {
        if !liquidity_sources.contains(&o.debtor_id) {
            for ls in liquidity_sources {
                let acceptance = ObligatoObligation {
                    id: Default::default(),
                    debtor_id: o.creditor_id,
                    creditor_id: *ls,
                    amount: u32::MAX as u64,
                };
                acc.insert(acceptance);
            }
        }
        acc
    });

    obligations.extend(acceptances.into_iter().collect::<Vec<_>>());
}

fn read_keys_file(keys_file: PathBuf) -> anyhow::Result<HashMap<Uuid, XPrv>> {
    let keys_file = File::open(keys_file)?;
    let keys_reader = BufReader::new(keys_file);
    let keys: HashMap<Uuid, String> = serde_json::from_reader(keys_reader)?;
    Ok(keys
        .into_iter()
        .map(|(id, key_str)| (id, XPrv::from_str(&key_str).unwrap()))
        .collect())
}

fn write_keys_to_file(output_file: PathBuf, keys: &HashMap<Uuid, XPrv>) {
    let keys_str: HashMap<_, _> = keys
        .iter()
        .map(|(id, k)| (id, k.to_string(Prefix::XPRV).to_string()))
        .collect();

    let output_file = File::create(output_file).expect("create file");
    let mut output_reader = BufWriter::new(output_file);
    output_reader
        .write_all(serde_json::to_string(&keys_str).unwrap().as_bytes())
        .expect("write file");
}

fn read_obligation_user_map_file(
    file: PathBuf,
) -> anyhow::Result<HashMap<HexBinary, (Uuid, Uuid)>> {
    let map_file = File::open(file)?;
    let map_reader = BufReader::new(map_file);
    serde_json::from_reader(map_reader).map_err(Into::into)
}

fn write_obligation_user_map_to_file(
    output_file: PathBuf,
    obligation_user_map: &HashMap<HexBinary, (Uuid, Uuid)>,
) {
    let output_file = File::create(output_file).expect("create file");
    let mut output_reader = BufWriter::new(output_file);
    output_reader
        .write_all(
            serde_json::to_string(&obligation_user_map)
                .unwrap()
                .as_bytes(),
        )
        .expect("write file");
}

fn derive_keys(
    obligations: &mut Vec<ObligatoObligation>,
    liquidity_sources: &[Uuid],
) -> anyhow::Result<HashMap<Uuid, XPrv>> {
    // Derive a BIP39 seed value using the given password
    let seed = global_seed()?;

    obligations.sort_by_key(|o| o.debtor_id);

    let mut keys = HashMap::new();

    for ls in liquidity_sources {
        keys.entry(*ls)
            .or_insert_with(|| derive_child_xprv(&seed, *ls));
    }

    for o in obligations {
        keys.entry(o.debtor_id)
            .or_insert_with(|| derive_child_xprv(&seed, o.debtor_id));
        keys.entry(o.creditor_id)
            .or_insert_with(|| derive_child_xprv(&seed, o.creditor_id));
    }

    Ok(keys)
}

fn derive_child_xprv(seed: &Seed, uuid: Uuid) -> XPrv {
    // Hash the UUID using SHA-256
    let mut hasher = Sha256::new();
    hasher.update(uuid.as_bytes());
    let uuid_digest = hasher.finalize();

    // Convert the hash bytes to a number
    let uuid_digest_num = u128::from_be_bytes(uuid_digest[..16].try_into().unwrap());

    // Take modulo (2^31 - 1)
    let address_index = uuid_digest_num % ((1u128 << 31) - 1);

    let child_path = format!("m/0/44'/118'/0'/0/{address_index}")
        .parse()
        .unwrap();
    let child_xprv = XPrv::derive_from_path(seed, &child_path);
    child_xprv.unwrap()
}

#[cfg(test)]
mod tests {
    use std::{error::Error, str::FromStr};

    use bip32::{Mnemonic, Prefix, PrivateKey, XPrv};
    use rand_core::OsRng;
    use uuid::Uuid;

    use crate::{derive_child_xprv, global_seed};

    #[test]
    fn test_create_mnemonic() {
        // Generate random Mnemonic using the default language (English)
        let mnemonic = Mnemonic::random(&mut OsRng, Default::default());
        println!("{}", mnemonic.phrase());
    }

    #[test]
    fn test_enc_dec_for_derived() -> Result<(), Box<dyn Error>> {
        let seed = global_seed()?;

        let alice_uuid = Uuid::from_u128(1);
        let alice_sk = derive_child_xprv(&seed, alice_uuid);
        let alice_pk = alice_sk.private_key().public_key();

        assert_eq!(
            alice_pk.to_sec1_bytes(),
            hex::decode("0219b0b8ee5fe9b317b69119fd15170d79737380c4f020e251b7839096f5513ccf")
                .unwrap()
                .into()
        );

        let alice_sk_str = alice_sk.to_string(Prefix::XPRV).to_string();
        assert_eq!(XPrv::from_str(&alice_sk_str).unwrap(), alice_sk);

        let msg = r#"{"debtor":"02027e3510f66f1f6c1ea5e3600062255928e518220f7883810cac3fc7fc092057","creditor":"0216254f4636c4e68ae22d98538851a46810b65162fe37bf57cba6d563617c913e","amount":10,"salt":"65c188bcc133add598f7eecc449112f4bf61024345316cff0eb5ce61291991b141073dcd3c543ea142e66fffa8f483dc382043d37e490ef9b8069c489ce94a0b"}"#;
        let ciphertext = ecies::encrypt(&alice_pk.to_sec1_bytes(), msg.as_bytes()).unwrap();
        // println!("{}", hex::encode(&ciphertext));

        let msg_dec =
            ecies::decrypt(&alice_sk.private_key().to_bytes(), ciphertext.as_slice()).unwrap();
        assert_eq!(msg, String::from_utf8(msg_dec).unwrap().as_str());

        Ok(())
    }
}


fn wasmaddr_to_id(address_str: &str) -> anyhow::Result<AccountId> {
    let (hr, _) = bech32_decode(address_str).map_err(|e| anyhow!(e))?;
    if hr != ADDRESS_PREFIX {
        return Err(anyhow!(hr));
    }

    Ok(address_str.parse().unwrap())
}