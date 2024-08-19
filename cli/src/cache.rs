use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};
use tracing::debug;
use xxhash_rust::xxh3::Xxh3;

use crate::error::Error;

const BUFFER_SIZE: usize = 16384; // 16 KB buffer
type Hash = u64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct DeployedContract {
    code_id: u64,
    contract_hash: Hash,
}

// Porcelain

pub async fn has_changed(file: &Path) -> Result<bool, Error> {
    let cur_hash: Hash = gen_hash(file).await?;
    debug!("current file hash: {}", cur_hash);

    let cached_file_path = to_cache_path(file)?;

    if !cached_file_path.exists() {
        return Ok(true);
    }

    let cached_contract = read_from_cache(cached_file_path.as_path()).await?;
    debug!("cached file hash: {}", cached_contract.contract_hash);

    Ok(cur_hash != cached_contract.contract_hash)
}

/// Return a hash of the given file's contents
pub async fn gen_hash(file: &Path) -> Result<Hash, Error> {
    let file = File::open(file)
        .await
        .map_err(|e| Error::GenericErr(e.to_string()))?;
    let mut reader = BufReader::new(file);

    let mut hasher = Xxh3::new();

    let mut buffer = [0; BUFFER_SIZE];
    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .await
            .map_err(|e| Error::GenericErr(e.to_string()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    // Finalize the hash
    let hash = hasher.digest();

    Ok(hash)
}

pub async fn save_codeid_to_cache(file: &Path, code_id: u64) -> Result<(), Error> {
    let contract_hash = gen_hash(file).await?;
    let dest = to_cache_path(file)?;
    let deployed_contract = DeployedContract {
        code_id,
        contract_hash,
    };

    write_to_cache(dest.as_path(), &deployed_contract).await
}

pub async fn get_cached_codeid(file: &Path) -> Result<u64, Error> {
    let cache_path = to_cache_path(file)?;
    let code_id = read_from_cache(cache_path.as_path()).await?.code_id;

    Ok(code_id)
}

// Plumbing

fn to_cache_path(file: &Path) -> Result<PathBuf, Error> {
    // Get cache filepath (".quartz/cache/example.wasm.json") from "example.wasm" filepath
    let mut filename = file.file_name().unwrap().to_os_string();
    filename.push(".json");

    let cached_file_path = cache_dir()?.join::<PathBuf>(filename.into());

    Ok(cached_file_path)
}

/// Retreive hash from cache file
async fn read_from_cache(cache_file: &Path) -> Result<DeployedContract, Error> {
    let content = tokio::fs::read_to_string(cache_file)
        .await
        .map_err(|e| Error::GenericErr(e.to_string()))?;
    serde_json::from_str(&content).map_err(|e| Error::GenericErr(e.to_string()))
}

/// Write a given file's contents hash to a file in cache directory
async fn write_to_cache(cache_file: &Path, data: &DeployedContract) -> Result<(), Error> {
    let content = serde_json::to_string(data).map_err(|e| Error::GenericErr(e.to_string()))?;
    tokio::fs::write(cache_file, content)
        .await
        .map_err(|e| Error::GenericErr(e.to_string()))
}

pub fn cache_dir() -> Result<PathBuf, Error> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| Error::GenericErr("Failed to grab home directory".to_string()))?;

    Ok(home_dir.join(".quartz/cache/"))
}

pub async fn create_cache_dir() -> Result<(), Error> {
    let cache_dir = cache_dir()?;
    if !cache_dir.exists() {
        tokio::fs::create_dir_all(&cache_dir)
            .await
            .map_err(|e| Error::GenericErr(e.to_string()))?;
    }

    Ok(())
}

// todo: rename these functions
pub fn log_dir() -> Result<PathBuf, Error> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| Error::GenericErr("Failed to grab home directory".to_string()))?;

    Ok(home_dir.join(".quartz/log/"))
}

pub async fn create_log_dir() -> Result<(), Error> {
    let log_dir = log_dir()?;
    if !log_dir.exists() {
        tokio::fs::create_dir_all(&log_dir)
            .await
            .map_err(|e| Error::GenericErr(e.to_string()))?;
    }

    Ok(())
}

pub async fn log_build_to_cache(build_dir_path: &Path) -> Result<(), Error> {
    let log_dir = log_dir()?;

    let app_name = build_dir_path.parent().unwrap().file_name().unwrap();
    let quartz_package = build_dir_path
        .file_name()
        .expect("function calls to this should have contract or enclave at the end");

    let filename = format!(
        "{}-{}",
        app_name.to_string_lossy(),
        quartz_package.to_string_lossy()
    );

    tokio::fs::write(log_dir.join(filename), "test")
        .await
        .map_err(|e| Error::GenericErr(e.to_string()))?;

    Ok(())
}
