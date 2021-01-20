use log::info;

use openssl::rand::rand_bytes;
use openssl::sign::{Signer, Verifier};
use openssl::rsa::{Rsa, Padding};
use openssl::pkey::{PKey, Public, Private};
use openssl::hash::MessageDigest;
use openssl::symm::{decrypt, encrypt, Cipher};

#[derive(Debug)]
struct KeyNotFoundError {}

impl std::fmt::Display for KeyNotFoundError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "private key not found")
  }
}

impl Error for KeyNotFoundError {
  fn description(&self) -> &str {
    "private key not found"
  }
}

fn look_for_key() -> Result<String, Box<dyn Error>> {
  let mut exe = std::env::current_exe()?;
  exe.pop();
  let parent = exe.as_path();

  info!("look for key in: {:?}", parent);

  for entry in parent.read_dir()? {
    let e = entry?;
    let name = e.file_name().into_string().unwrap();
    if name.ends_with(".key") {
      info!("found key: {}", name);
      return Ok(name)
    }
  }

  Err(Box::new(KeyNotFoundError{}))
}

pub struct User {
  cert: PKey<Private>,
}

impl User {
  pub fn new () -> User {
    User {}
  }
}