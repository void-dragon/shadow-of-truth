use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use tokio::sync::mpsc::Sender;

use shadow_of_truth_common::Message;

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

pub enum ClientState {
  Greeting,
  SecretSharing,
  Listening,
  Disconnected,
}

pub struct Client {
  pub id: String,
  pub room: String,
  pub owned_spawns: HashSet<String>,
  pub state: ClientState,
  pub tx: Sender<Message>,
}

impl PartialEq for Client {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl Eq for Client {}

impl Hash for Client {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.id.hash(state);
  }
}