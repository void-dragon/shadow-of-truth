use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use shadow_of_truth_common as common;

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

    self.send(common::Message::Login);

    std::thread::spawn(move || {
      loop {
        match common::read(&mut reader) {
          Ok(msg) => {}
          Err(e) => {
            log::error!("read {}", e.to_string());
            break;
          }
        }
      }
    });

    Ok(())
  }

  pub fn send(&self, msg: common::Message) {
    // if let Some(ref writer) = self.writer {
    //   let mut writer = writer.lock().unwrap();
    //   if let Err(e) = common::write(&mut *writer, msg) {
    //     log::error!("write {}", e.to_string());
    //   }
    // }
  }
}

pub fn new() -> Network {
  Network {
    writer: None,
  }
}