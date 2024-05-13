#![forbid(unsafe_code)]
#![warn(
    clippy::checked_conversions,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_used,
    trivial_casts,
    trivial_numeric_casts,
    rust_2018_idioms,
    unused_lifetimes,
    unused_import_braces,
    unused_qualifications
)]

use std::{
    error::Error,
    fs::{read_to_string, File},
    io::Write,
    path::PathBuf,
    str::FromStr,
};

use bip32::XPrv;
use clap::{Parser, Subcommand};
use cosmrs::{tendermint::account::Id as TmAccountId, AccountId};
use cosmwasm_std::HexBinary;
use ecies::{decrypt, encrypt};
use k256::{
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::generic_array::GenericArray,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
enum Command {
    KeyGen {
        #[clap(long, default_value = "epoch.pk")]
        pk_file: PathBuf,
        #[clap(long, default_value = "epoch.sk")]
        sk_file: PathBuf,
    },
    EncryptObligation {
        #[clap(long, value_parser = parse_obligation_json)]
        obligation: Obligation,
        #[clap(long, default_value = "epoch.pk")]
        pk_file: PathBuf,
    },
    DecryptObligation {
        #[clap(long)]
        obligation: String,
        #[clap(long, default_value = "epoch.sk")]
        sk_file: PathBuf,
    },
    EncryptSetoff {
        #[clap(long, value_parser = parse_setoff_json)]
        setoff: Setoff,
        #[clap(long)]
        obligation_digest: String,
        #[clap(long, default_value = "user.pk")]
        pk_file: PathBuf,
    },
    DecryptSetoff {
        #[clap(long)]
        setoff: String,
        #[clap(long, default_value = "user.sk")]
        sk_file: PathBuf,
    },
    PrintAddress {
        #[clap(long)]
        pk: String,
    },
    PrintAddressFromPriv {
        #[clap(long)]
        sk_str: String,
    },
}

fn parse_obligation_json(s: &str) -> Result<Obligation, String> {
    let raw_obligation: RawObligation = serde_json::from_str(s).map_err(|e| e.to_string())?;
    raw_obligation.try_into()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RawObligation {
    debtor: HexBinary,
    creditor: HexBinary,
    amount: u64,
    #[serde(default)]
    salt: HexBinary,
}

#[derive(Clone, Debug)]
struct Obligation {
    debtor: VerifyingKey,
    creditor: VerifyingKey,
    amount: u64,
    salt: [u8; 64],
}

impl TryFrom<RawObligation> for Obligation {
    type Error = String;

    fn try_from(raw_obligation: RawObligation) -> Result<Self, Self::Error> {
        let mut salt = [0u8; 64];
        rand::thread_rng().fill(&mut salt[..]);

        Ok(Self {
            debtor: VerifyingKey::from_sec1_bytes(raw_obligation.debtor.as_slice())
                .map_err(|e| e.to_string())?,
            creditor: VerifyingKey::from_sec1_bytes(raw_obligation.creditor.as_slice())
                .map_err(|e| e.to_string())?,
            amount: raw_obligation.amount,
            salt,
        })
    }
}

impl From<Obligation> for RawObligation {
    fn from(obligation: Obligation) -> Self {
        Self {
            debtor: obligation.debtor.to_sec1_bytes().into_vec().into(),
            creditor: obligation.creditor.to_sec1_bytes().into_vec().into(),
            amount: obligation.amount,
            salt: obligation.salt.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct EncryptedIntent {
    ciphertext: HexBinary,
    digest: HexBinary,
}

fn parse_setoff_json(s: &str) -> Result<Setoff, String> {
    let raw_setoff: RawSetoff = serde_json::from_str(s).map_err(|e| e.to_string())?;
    raw_setoff.try_into()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RawSetoff {
    debtor: HexBinary,
    creditor: HexBinary,
    amount: u64,
    #[serde(default)]
    salt: HexBinary,
}

#[derive(Clone, Debug)]
struct Setoff {
    debtor: VerifyingKey,
    creditor: VerifyingKey,
    amount: u64,
    salt: [u8; 64],
}

impl TryFrom<RawSetoff> for Setoff {
    type Error = String;

    fn try_from(raw_setoff: RawSetoff) -> Result<Self, Self::Error> {
        let mut salt = [0u8; 64];
        rand::thread_rng().fill(&mut salt[..]);

        Ok(Self {
            debtor: VerifyingKey::from_sec1_bytes(raw_setoff.debtor.as_slice())
                .map_err(|e| e.to_string())?,
            creditor: VerifyingKey::from_sec1_bytes(raw_setoff.creditor.as_slice())
                .map_err(|e| e.to_string())?,
            amount: raw_setoff.amount,
            salt,
        })
    }
}

impl From<Setoff> for RawSetoff {
    fn from(setoff: Setoff) -> Self {
        Self {
            debtor: setoff.debtor.to_sec1_bytes().into_vec().into(),
            creditor: setoff.creditor.to_sec1_bytes().into_vec().into(),
            amount: setoff.amount,
            salt: setoff.salt.into(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    match args.command {
        Command::KeyGen { pk_file, sk_file } => {
            let sk = SigningKey::random(&mut rand::thread_rng());
            let pk = sk.verifying_key();

            let mut sk_file = File::create(sk_file)?;
            let sk = hex::encode(sk.to_bytes());
            sk_file.write_all(sk.as_bytes())?;

            let mut pk_file = File::create(pk_file)?;
            let pk = hex::encode(pk.to_sec1_bytes());
            pk_file.write_all(pk.as_bytes())?;
        }
        Command::EncryptObligation {
            obligation,
            pk_file,
        } => {
            let epoch_pk = {
                let pk_str = read_to_string(pk_file)?;
                hex::decode(pk_str)?
            };
            let obligation_ser = serde_json::to_string(&RawObligation::from(obligation))
                .expect("infallible serializer");

            let ciphertext =
                encrypt(&epoch_pk, obligation_ser.as_bytes()).map_err(|e| e.to_string())?;

            let digest: [u8; 32] = {
                let mut hasher = Sha256::new();
                hasher.update(obligation_ser);
                hasher.finalize().into()
            };

            let obligation_enc = EncryptedIntent {
                ciphertext: ciphertext.into(),
                digest: digest.into(),
            };

            println!(
                "{}",
                serde_json::to_string(&obligation_enc).expect("infallible serializer")
            );
        }
        Command::DecryptObligation {
            obligation,
            sk_file,
        } => {
            let sk = {
                let sk_str = read_to_string(sk_file)?;
                let sk = hex::decode(sk_str).expect("");
                SigningKey::from_bytes(GenericArray::from_slice(&sk))?
            };

            let ciphertext = hex::decode(obligation).unwrap();

            let obligation = {
                let o = decrypt(&sk.to_bytes(), &ciphertext).unwrap();
                serde_json::from_slice::<RawObligation>(&o)?
            };
            println!("{obligation:?}");
        }
        Command::EncryptSetoff {
            setoff,
            obligation_digest,
            pk_file,
        } => {
            let pk = {
                let pk_str = read_to_string(pk_file)?;
                hex::decode(pk_str)?
            };
            let setoff_ser =
                serde_json::to_string(&RawSetoff::from(setoff)).expect("infallible serializer");

            let ciphertext = encrypt(&pk, setoff_ser.as_bytes()).map_err(|e| e.to_string())?;

            let digest: [u8; 32] = {
                let d = hex::decode(obligation_digest)?;
                d.try_into().unwrap()
            };

            let setoff_enc = EncryptedIntent {
                ciphertext: ciphertext.into(),
                digest: digest.into(),
            };

            println!(
                "{}",
                serde_json::to_string(&setoff_enc).expect("infallible serializer")
            );
        }
        Command::DecryptSetoff { setoff, sk_file } => {
            let sk = {
                let sk_str = read_to_string(sk_file)?;
                let sk = hex::decode(sk_str).expect("");
                SigningKey::from_bytes(GenericArray::from_slice(&sk))?
            };

            let ciphertext = hex::decode(setoff).unwrap();

            let setoff = decrypt(&sk.to_bytes(), &ciphertext).unwrap();
            serde_json::from_slice(&setoff)?;
        }
        Command::PrintAddress { pk } => {
            let pk = {
                let pk = hex::decode(pk)?;
                VerifyingKey::from_sec1_bytes(&pk)?
            };
            let tm_pk = TmAccountId::from(pk);
            println!("{}", AccountId::new("wasm", tm_pk.as_bytes()).unwrap());
        }
        Command::PrintAddressFromPriv { sk_str } => {
            let sk = XPrv::from_str(&sk_str).unwrap();

            let pk_b = sk.public_key().public_key().to_sec1_bytes();
            let pk = VerifyingKey::from_sec1_bytes(&pk_b)?;

            let pk_bytes = hex::encode(pk.to_sec1_bytes());

            println!("{}", pk_bytes);

            let tm_pk = TmAccountId::from(pk);
            println!("{}", AccountId::new("wasm", tm_pk.as_bytes()).unwrap());
        }
    }

    Ok(())
}
