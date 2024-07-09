use std::env;

fn main() {
    let ra_type = env::var("RA_TYPE").unwrap_or_else(|_| "epid".to_string());
    println!("cargo:rustc-cfg=ra_type=\"{}\"", ra_type);
}
