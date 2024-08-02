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
    Error as Bip32Error, Language, Mnemonic, Prefix, PrivateKey, Seed, XPrv,
};
use clap::Parser;
use cosmrs::{tendermint::account::Id as TmAccountId, AccountId};
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
        RawOffset, RawSetOff, SubmitObligatioMsg, SubmitObligatoObligationsMsgInner,
    },
    wasmd_client::{CliWasmdClient, QueryResult, WasmdClient},
};

mod cli;
mod obligato_client;
mod types;
mod wasmd_client;

const MNEMONIC_PHRASE: &str = "clutch debate vintage foster barely primary clown leader sell manual leopard ladder wet must embody story oyster imitate cable alien six square rice wedding";

const ADDRESS_PREFIX: &str = "wasm";

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
        CliCommand::SyncObligations {
            ref epoch_pk,
            ref liquidity_sources,
        } => sync_obligations(cli.clone(), epoch_pk, liquidity_sources).await?,
        CliCommand::SyncSetOffs => sync_setoffs(cli).await?,
        CliCommand::GetAddress { uuid } => address_from_uuid(uuid)?,
    }

    Ok(())
}

fn address_from_uuid(uuid: Uuid) -> Result<(), DynError> {
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

async fn sync_setoffs(cli: Cli) -> Result<(), DynError> {
    let wasmd_client = CliWasmdClient::new(cli.node);
    let query_result: QueryResult<QueryAllSetoffsResponse> =
        wasmd_client.query_smart(&cli.contract, json!("get_all_setoffs"))?;
    let setoffs = query_result.data.setoffs;

    // read keys
    let keys = read_keys_file(cli.keys_file)?;
    let obligation_user_map = read_obligation_user_map_file(cli.obligation_user_map_file)?;

    let setoffs: Vec<ObligatoSetOff> = setoffs
        .iter()
        .flat_map(|(obligation_digest, so)| match so {
            RawSetOff::SetOff(sos_enc) => {
                let so_enc = sos_enc.first().unwrap();
                let (debtor_id, creditor_id) =
                    obligation_user_map.get(obligation_digest).copied().unwrap();

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
    let client = HttpClient::new(cli.obligato_url, cli.obligato_key);
    client.set_setoffs(setoffs).await?;

    Ok(())
}

async fn sync_obligations(
    cli: Cli,
    epoch_pk: &str,
    liquidity_sources: &[Uuid],
) -> Result<(), DynError> {
    let mut intents = {
        let client = HttpClient::new(cli.obligato_url.clone(), cli.obligato_key);
        client
            .get_obligations()
            .await
            .map_err(|_| cli.obligato_url.to_string())?
    };

    let keys = derive_keys(&mut intents, liquidity_sources)?;
    write_keys_to_file(cli.keys_file, &keys);

    add_default_acceptances(&mut intents, liquidity_sources);

    debug!("intents: {intents:?}");

    let intents_enc = {
        let epoch_pk = VerifyingKey::from_sec1_bytes(&hex::decode(epoch_pk).unwrap()).unwrap();
        encrypt_intents(intents, &keys, &epoch_pk, cli.obligation_user_map_file)
    };
    debug!("Encrypted {} intents", intents_enc.len());

    let liquidity_sources = liquidity_sources
        .iter()
        .map(|id| keys[id].private_key().public_key())
        .collect();

    let msg = create_wasm_msg(intents_enc, liquidity_sources)?;
    let wasmd_client = CliWasmdClient::new(cli.node);
    wasmd_client.tx_execute(&cli.contract, &cli.chain_id, 3000000, cli.user, msg)?;

    Ok(())
}

fn create_wasm_msg(
    obligations_enc: Vec<(Sha256Digest, Vec<u8>)>,
    liquidity_sources: Vec<VerifyingKey>,
) -> Result<serde_json::Value, DynError> {
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
        .map(|pk| HexBinary::from(pk.to_sec1_bytes().as_ref()))
        .collect();

    let msg = SubmitObligatioMsg {
        submit_obligations: SubmitObligatoObligationsMsgInner {
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
    liquidity_sources: &[Uuid],
) -> Result<HashMap<Uuid, XPrv>, DynError> {
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
        let mnemonic = Mnemonic::random(OsRng, Default::default());
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
