use std::{
    fs::File,
    io::{self, Read},
};

fn main() -> io::Result<()> {
    let mut file = File::open("/dev/attestation")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let quote_hex = hex::encode(&buffer);
    println!("{}", quote_hex);

    Ok(())
}
