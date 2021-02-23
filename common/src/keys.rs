use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

use openssl::rsa::{Rsa, Padding};
use openssl::pkey::{PKey, Public, Private};

type PrivateResult = Result<PKey<Private>, Box<dyn Error>>;

pub fn optain_private_key(path: &std::path::Path) -> PrivateResult {
  if path.exists() {
    load_key(path)
  }
  else {
    generate_key(path)
  }
}

fn generate_key(path: &std::path::Path) -> PrivateResult {
  log::info!("generate private key");
  let key = Rsa::generate(4096)?;
  let pkey = PKey::from_rsa(key)?;

  let data = pkey.private_key_to_pem_pkcs8()?;
  let mut file = File::create(path)?;
  file.write(&data)?;

  Ok(pkey)
}

fn load_key(path: &std::path::Path) -> PrivateResult {
  log::info!("load private key");
  let mut file = File::open(path)?;
  let mut buffer = Vec::new();

  file.read_to_end(&mut buffer)?;

  Ok(PKey::private_key_from_pem(&buffer)?)
}