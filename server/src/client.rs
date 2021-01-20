use tokio::sync::mpsc::Sender;

use crate::message::Command;

pub struct AesKey {
  pub key: [u8; 32],
  pub iv: [u8; 16],
}

impl AesKey {
  fn new(key: Vec<u8>, iv: Vec<u8>) -> AesKey {
    let mut a = AesKey {
      key: [0; 32],
      iv: [0; 16],
    };

    a.key.copy_from_slice(&key);
    a.iv.copy_from_slice(&iv);

    a
  }
}

enum ClientState {
  Greeting,
  SecretSharing,
  Listening,
}

pub struct Client {
  state: ClientState,
  tx: Sender<Command>,
}