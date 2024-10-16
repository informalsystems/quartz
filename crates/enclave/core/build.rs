use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let source_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../../../crates/contracts/tee-ra/data");

    fs::create_dir_all(&out_dir).unwrap();

    let files_to_copy = [
        "qe_identity.json",
        "root_ca.pem",
        "root_crl.der",
        "tcb_signer.pem",
    ];

    for file in &files_to_copy {
        let source_path = source_dir.join(file);
        let target_path = out_dir.join(file);

        fs::copy(&source_path, &target_path)
            .unwrap_or_else(|_| panic!("Failed to copy {:?}", source_path));
    }
}
