use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};
use tracing::debug;
use xxhash_rust::xxh3::Xxh3;

use color_eyre::{eyre::eyre, Report, Result};
use crate::config::Config;

const BUFFER_SIZE: usize = 16384; // 16 KB buffer
type Hash = u64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct DeployedContract {
    code_id: u64,
    contract_hash: Hash,
}

// Porcelain

impl Config {
    pub async fn contract_has_changed(&self, file: &Path) -> Result<bool> {
        let cur_hash: Hash = Self::gen_hash(file).await?;
        debug!("current file hash: {}", cur_hash);

        let cached_file_path = Self::to_cache_path(self, file)?;

        if !cached_file_path.exists() {
            return Ok(true);
        }

        let cached_contract = Self::read_from_cache(cached_file_path.as_path()).await?;
        debug!("cached file hash: {}", cached_contract.contract_hash);

        Ok(cur_hash != cached_contract.contract_hash)
    }

    /// Return a hash of the given file's contents
    pub async fn gen_hash(file: &Path) -> Result<Hash> {
        let file = File::open(file)
            .await?;

        let mut reader = BufReader::new(file);

        let mut hasher = Xxh3::new();

        let mut buffer = [0; BUFFER_SIZE];
        loop {
            let bytes_read = reader
                .read(&mut buffer)
                .await?;
            
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        // Finalize the hash
        let hash = hasher.digest();

        Ok(hash)
    }

    pub async fn save_codeid_to_cache(&self, file: &Path, code_id: u64) -> Result<()> {
        let contract_hash = Self::gen_hash(file).await?;
        let dest = Self::to_cache_path(self, file)?;
        let deployed_contract = DeployedContract {
            code_id,
            contract_hash,
        };

        Self::write_to_cache(dest.as_path(), &deployed_contract).await
    }

    pub async fn get_cached_codeid(&self, file: &Path) -> Result<u64> {
        let cache_path = Self::to_cache_path(self, file)?;
        let code_id = Self::read_from_cache(cache_path.as_path()).await?.code_id;

        Ok(code_id)
    }

    // Plumbing

    fn to_cache_path(&self, file: &Path) -> Result<PathBuf> {
        // Get cache filepath (".quartz/cache/example.wasm.json") from "example.wasm" filepath
        let mut filename = file
            .file_name()
            .ok_or(eyre!("file at cache filepath does not exist {}", file.display()))?
            .to_os_string();

        filename.push(".json");

        let cached_file_path = Self::cache_dir(self)?.join::<PathBuf>(filename.into());

        Ok(cached_file_path)
    }

    /// Retreive hash from cache file
    async fn read_from_cache(cache_file: &Path) -> Result<DeployedContract> {
        let content = tokio::fs::read_to_string(cache_file)
            .await?;

        serde_json::from_str(&content).map_err(|e| eyre!(e))
    }

    /// Write a given file's contents hash to a file in cache directory
    async fn write_to_cache(cache_file: &Path, data: &DeployedContract) -> Result<()> {
        let content = serde_json::to_string(data)?;

        tokio::fs::write(cache_file, content)
            .await
            .map_err(|e| eyre!(e))
    }

    pub fn cache_dir(&self) -> Result<PathBuf> {
        Ok(self.app_dir.join(".cache/"))
    }

    pub fn build_log_dir(&self) -> Result<PathBuf> {
        Ok(self.app_dir.join(".cache/log/"))
    }

    /// Creates the build log if it isn't created already, returns relative path from app_dir to log directory
    pub async fn create_build_log(&self) -> Result<PathBuf> {
        let log_dir = Self::build_log_dir(self)?;
        if !log_dir.exists() {
            tokio::fs::create_dir_all(&log_dir)
                .await?;
        }

        Ok(log_dir)
    }

    pub async fn log_build(&self, is_enclave: bool) -> Result<()> {
        let log_dir = Self::create_build_log(self).await?;

        let filename = match is_enclave {
            true => "enclave",
            false => "contract",
        };

        tokio::fs::write(log_dir.join(filename), "test")
            .await?;

        Ok(())
    }
}
