use std::{
    fs, io,
    path::{Path, PathBuf},
};

use tracing::trace;
use walkdir::WalkDir;

use crate::{
    cli::Verbosity,
    error::Error,
    handler::Handler,
    request::init::InitRequest,
    response::{init::InitResponse, Response},
};

impl Handler for InitRequest {
    type Error = Error;
    type Response = Response;

    fn handle(self, _verbosity: Verbosity) -> Result<Self::Response, Self::Error> {
        trace!("initializing directory structure...");

        let cli_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let example_dir = cli_manifest_dir.join("example");

        copy_dir_recursive(example_dir.as_path(), self.directory.as_path())
            .map_err(|e| Error::GenericErr(e.to_string()))?;

        Ok(InitResponse.into())
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    // Create the destination directory if it doesn't exist
    fs::create_dir_all(dst)?;

    for entry in WalkDir::new(src) {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(src).map_err(io::Error::other)?;
        let target = dst.join(relative);

        if path.is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            fs::copy(path, &target)?;
        }
    }

    Ok(())
}
