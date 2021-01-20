use std::net::TcpStream;
use std::io::{Read, Write};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum Message {
  Login
}

#[derive(Clone)]
pub struct Network {
  writer: Option<Arc<Mutex<TcpStream>>>,
}

impl Network {
  pub fn establish_connection(&self) {
    let mut net = self.clone();

    std::thread::spawn(move || {
      let mut count = 0;

      while let Err(e) = net.try_connect() {
        log::warn!("{}", e.to_string());

        count += 1;

        if count > 3 {
          log::error!("could not establish connection");
          return;
        }

        std::thread::sleep(Duration::from_secs(10));
      }
    });
  }

  pub fn try_connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = TcpStream::connect("127.0.0.1:3000")?;
    self.writer = Some(Arc::new(Mutex::new(reader.try_clone()?)));

    self.send(Message::Login);

    std::thread::spawn(move || {
      loop {
        let mut size_buffer = [0u8; 4];
        reader.read_exact(&mut size_buffer).unwrap();

        let size = u32::from_le_bytes(size_buffer);
        let mut data = vec![0u8; size as usize];
        reader.read_exact(&mut data).unwrap();

        let msg: Message = serde_cbor::from_slice(&data).unwrap();
      }
    });

    Ok(())
  }

  pub fn send(&self, msg: Message) {
    if let Some(ref writer) = self.writer {
      let mut writer = writer.lock().unwrap();
      let msg = serde_cbor::to_vec(&msg).unwrap();
      let size_buffer = (msg.len() as u32).to_le_bytes();

      writer.write(&size_buffer).unwrap();
      writer.write(&msg).unwrap();
    }
  }
}

pub fn new() -> Network {
  Network {
    writer: None,
  }
}