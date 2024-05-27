use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
    str::FromStr,
};

use bip32::{
    secp256k1::{
        ecdsa::VerifyingKey,
        sha2::{Digest, Sha256},
    },
    Language, Mnemonic, Prefix, PrivateKey, Seed, XPrv,
};
use clap::Parser;
use cosmwasm_std::HexBinary;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, Level};
use uuid::Uuid;

use crate::{
    cli::{Cli, CliCommand},
    obligato_client::{http::HttpClient, Client},
    types::{
        Obligation, ObligatoObligation, ObligatoSetOff, RawEncryptedObligation, RawObligation,
        RawOffset, RawSetOff, SubmitObligationsMsg,
    },
    wasmd_client::{CliWasmdClient, QueryResult, WasmdClient},
};

mod cli;
mod obligato_client;
mod types;
mod wasmd_client;

const MNEMONIC_PHRASE: &str = "clutch debate vintage foster barely primary clown leader sell manual leopard ladder wet must embody story oyster imitate cable alien six square rice wedding";

const ALICE_ID: &str = "7bfad4e8-d898-4ce2-bbac-1beff7182319";
const BANK_DEBTOR_ID: &str = "3879fa15-d86e-4464-b679-0a3d78cf3dd3";

const OBLIGATO_URL: &str = "https://deploy-preview-353--obligato-app-bisenzone.netlify.app";
const OBLIGATO_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImRydXZveWVhYXN5bXZubGxmdnZ5Iiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImlhdCI6MTcxMTYyNDgzNiwiZXhwIjoyMDI3MjAwODM2fQ.y-2iTQCplrXBEzHrvz_ZGFmMx-iLMzRZ6I0N5htJ39c";

type Sha256Digest = [u8; 32];

type DynError = Box<dyn Error>;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct QueryAllSetoffsResponse {
    setoffs: Vec<(HexBinary, RawSetOff)>,
}

#[tokio::main]
async fn main() -> Result<(), DynError> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_max_level(if cli.verbose {
            Level::DEBUG
        } else {
            Level::ERROR
        })
        .with_level(true)
        .with_writer(std::io::stderr)
        .init();

    match cli.command {
        CliCommand::SyncObligations { ref epoch_pk } => {
            sync_obligations(cli.clone(), epoch_pk).await?
        }
        CliCommand::SyncSetOffs => sync_setoffs(cli).await?,
    }

    Ok(())
}

async fn sync_setoffs(cli: Cli) -> Result<(), DynError> {
    let wasmd_client = CliWasmdClient::new(cli.node);
    let query_result: QueryResult<QueryAllSetoffsResponse> =
        wasmd_client.query_smart(&cli.contract, json!("get_all_setoffs"), &cli.chain_id)?;
    let setoffs = query_result.data.setoffs;

    // read keys
    let keys = read_keys_file(cli.keys_file)?;
    let obligation_user_map = read_obligation_user_map_file(cli.obligation_user_map_file)?;

    let setoffs: Vec<ObligatoSetOff> = setoffs
        .iter()
        .flat_map(|(obligation_digest, so)| match so {
            RawSetOff::SetOff(sos_enc) => {
                let so_enc = sos_enc.first().unwrap();
                let (debtor_id, creditor_id) = obligation_user_map
                    .get(obligation_digest)
                    .map(Clone::clone)
                    .unwrap();

                let sk = |id| keys[&id].private_key().to_bytes();
                let so_ser = if let Ok(so) = ecies::decrypt(&sk(debtor_id), so_enc.as_slice()) {
                    so
                } else if let Ok(so) = ecies::decrypt(&sk(creditor_id), so_enc.as_slice()) {
                    so
                } else {
                    unreachable!()
                };

                let so: RawOffset = serde_json::from_slice(&so_ser).unwrap();
                Some(ObligatoSetOff {
                    debtor_id,
                    creditor_id,
                    amount: so.set_off,
                })
            }
            RawSetOff::Transfer(_) => None,
        })
        .collect();

    debug!("setoffs: {setoffs:?}");

    // send to Obligato
    let client = HttpClient::new(OBLIGATO_URL.parse().unwrap());
    client.set_setoffs(setoffs).await?;

    Ok(())
}

async fn sync_obligations(cli: Cli, epoch_pk: &str) -> Result<(), DynError> {
    let mut intents = {
        let client = HttpClient::new(OBLIGATO_URL.parse().unwrap());
        client.get_obligations().await.unwrap()
    };

    let bank_id = Uuid::parse_str(BANK_DEBTOR_ID).unwrap();
    let keys = derive_keys(&mut intents, bank_id)?;
    write_keys_to_file(cli.keys_file, &keys);

    add_default_acceptances(&mut intents, bank_id);

    debug!("intents: {intents:?}");

    let intents_enc = {
        let epoch_pk = VerifyingKey::from_sec1_bytes(&hex::decode(epoch_pk).unwrap()).unwrap();
        encrypt_intents(intents, keys, &epoch_pk, cli.obligation_user_map_file)
    };
    debug!("Encrypted {} intents", intents_enc.len());

    let msg = create_wasm_msg(intents_enc);
    let wasmd_client = CliWasmdClient::new(cli.node);
    wasmd_client.tx_execute(&cli.contract, &cli.chain_id, 3000000, cli.user, msg)?;

    Ok(())
}

fn create_wasm_msg(obligations_enc: Vec<(Sha256Digest, Vec<u8>)>) -> serde_json::Value {
    let obligations_enc: Vec<_> = obligations_enc
        .into_iter()
        .map(|(digest, ciphertext)| {
            let digest = HexBinary::from(digest);
            let ciphertext = HexBinary::from(ciphertext);
            RawEncryptedObligation { digest, ciphertext }
        })
        .collect();

    let msg = SubmitObligationsMsg {
        submit_obligations: obligations_enc,
    };
    serde_json::to_value(msg).unwrap()
}

fn encrypt_intents(
    intents: Vec<ObligatoObligation>,
    keys: HashMap<Uuid, XPrv>,
    epoch_pk: &VerifyingKey,
    obligation_user_map_file: PathBuf,
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

    write_obligation_user_map_to_file(obligation_user_map_file, &intent_user_map);

    intents_enc
}

fn add_default_acceptances(obligations: &mut Vec<ObligatoObligation>, bank_id: Uuid) {
    let acceptances = obligations.iter().fold(HashSet::new(), |mut acc, o| {
        if o.debtor_id != bank_id {
            let acceptance = ObligatoObligation {
                id: Default::default(),
                debtor_id: o.creditor_id,
                creditor_id: bank_id,
                amount: u32::MAX as u64,
            };
            acc.insert(acceptance);
        }
        acc
    });

    obligations.extend(acceptances.into_iter().collect::<Vec<_>>());
}

fn read_keys_file(keys_file: PathBuf) -> Result<HashMap<Uuid, XPrv>, DynError> {
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
) -> Result<HashMap<HexBinary, (Uuid, Uuid)>, DynError> {
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
    bank_id: Uuid,
) -> Result<HashMap<Uuid, XPrv>, DynError> {
    // Derive a BIP39 seed value using the given password
    let seed = {
        let mnemonic = Mnemonic::new(MNEMONIC_PHRASE, Language::English)?;
        mnemonic.to_seed("password")
    };

    obligations.sort_by_key(|o| o.debtor_id);

    let mut keys = HashMap::new();
    let mut child_num = 0;

    let alice_id = Uuid::parse_str(ALICE_ID).unwrap();

    keys.entry(alice_id)
        .or_insert_with(|| derive_child_xprv(&seed, &mut child_num));

    keys.entry(bank_id)
        .or_insert_with(|| derive_child_xprv(&seed, &mut child_num));

    for o in obligations {
        keys.entry(o.debtor_id)
            .or_insert_with(|| derive_child_xprv(&seed, &mut child_num));
        keys.entry(o.creditor_id)
            .or_insert_with(|| derive_child_xprv(&seed, &mut child_num));
    }

    Ok(keys)
}

fn derive_child_xprv(seed: &Seed, i: &mut usize) -> XPrv {
    let child_path = format!("m/0/44'/118'/0'/0/{}", i).parse().unwrap();
    let child_xprv = XPrv::derive_from_path(seed, &child_path);
    *i += 1;
    child_xprv.unwrap()
}

#[cfg(test)]
mod tests {
    use std::{error::Error, str::FromStr};

    use bip32::{Language, Mnemonic, Prefix, PrivateKey, XPrv};
    use rand_core::OsRng;

    use crate::{derive_child_xprv, MNEMONIC_PHRASE};

    #[test]
    fn test_create_mnemonic() {
        // Generate random Mnemonic using the default language (English)
        let mnemonic = Mnemonic::random(&mut OsRng, Default::default());

        println!("{}", mnemonic.phrase());
    }

    #[test]
    fn test_enc_dec_for_derived() -> Result<(), Box<dyn Error>> {
        let seed = {
            let mnemonic = Mnemonic::new(MNEMONIC_PHRASE, Language::English)?;
            mnemonic.to_seed("password")
        };

        let mut child_num = 0;
        let alice_sk = derive_child_xprv(&seed, &mut child_num);
        let alice_sk_str = alice_sk.to_string(Prefix::XPRV).to_string();
        assert_eq!(
            alice_sk.private_key().public_key().to_sec1_bytes(),
            hex::decode("02027e3510f66f1f6c1ea5e3600062255928e518220f7883810cac3fc7fc092057")
                .unwrap()
                .into()
        );
        assert_eq!(XPrv::from_str(&alice_sk_str).unwrap(), alice_sk);

        let alice_pk = alice_sk.private_key().public_key();
        assert_eq!(
            alice_pk.to_sec1_bytes().into_vec(),
            vec![
                2, 2, 126, 53, 16, 246, 111, 31, 108, 30, 165, 227, 96, 0, 98, 37, 89, 40, 229, 24,
                34, 15, 120, 131, 129, 12, 172, 63, 199, 252, 9, 32, 87
            ]
        );

        let msg = r#"{"debtor":"02027e3510f66f1f6c1ea5e3600062255928e518220f7883810cac3fc7fc092057","creditor":"0216254f4636c4e68ae22d98538851a46810b65162fe37bf57cba6d563617c913e","amount":10,"salt":"65c188bcc133add598f7eecc449112f4bf61024345316cff0eb5ce61291991b141073dcd3c543ea142e66fffa8f483dc382043d37e490ef9b8069c489ce94a0b"}"#;

        let ciphertext = ecies::encrypt(&alice_pk.to_sec1_bytes(), msg.as_bytes()).unwrap();
        // let ciphertext = hex::decode("0418d9051cbfc86c8ddd57ae43ea3d1ac8b30353a3ecd8c806bb11f0693dfd282d5f07d1de32cbcd933d5ab7cd0aa171c972e75531b915e968f0fdeba78fa3f359c7f3ef7ae2dfffeb19493e9b2418dc774e6e80448a2dc4a7ba657cd4a8456e120977ebe372a57187d53981cc5856fbd63e9c1bdf001ed71c3d50cbaff594561191d33dad852cb782126f480add2cc92758b59eb63de857d299eaa5f09fbc55643a73b1d8206ce83453b5296b566d9f622520679bb3e6d9c8b7a707f33d3093c41dfc0a8267749b4028e9ee0faad0c8df64f1682a348f220585fdd9b9ac411bdaaa6a249b45accc89a80e5af09abb239231aa869e29459e562721b685d98b3da3eeaef14e1c5f3bd20cf27c0cbbae7b5c618e737df9a84f9a040bb472b7254af2cf4ccc76784cf8432080e528f700ca2a082b7020d94f0f5325dd4998c03972a0b39e6670b65be89e7a80aad7af08a393fcf2e103999254380c1f0355d97ddcdfaeed4bcfaf15b578cee1f6d3fd4ceccd85760b9bd714f81698ddf6fbbc06152a9306a5dd0052c722e390470f0c70eeac81a5da0090").unwrap();

        println!("{}", hex::encode(&ciphertext));

        let msg_dec =
            ecies::decrypt(&alice_sk.private_key().to_bytes(), ciphertext.as_slice()).unwrap();
        assert_eq!(msg, String::from_utf8(msg_dec).unwrap().as_str());

        Ok(())
    }
}
