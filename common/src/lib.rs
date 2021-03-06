use std::io::{Read, Write, ErrorKind};

use serde::{Serialize, Deserialize};
use tokio::{
    io::{
      AsyncReadExt,
      AsyncWriteExt,
    },
    net::{
        tcp::{
          OwnedReadHalf,
          OwnedWriteHalf,
        },
    },
};

pub mod keys;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
  Login{id: String},
  Join{scene: String},
  Leave{scene: String},
  Spawn{id: String, scene: String, drawable: String, behavior: Option<String>},
  Destroy{id: String, scene: String},
  TransformUpdate{scene: String, id: String, t: [f32; 16]},
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Signed {
  #[serde(with = "serde_bytes")]
  pub sign: Vec<u8>,
  #[serde(with = "serde_bytes")]
  pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
  Key {
    #[serde(with = "serde_bytes")]
    key: Vec<u8>,
    #[serde(with = "serde_bytes")]
    iv: Vec<u8>,
  },
}

pub fn read<T: Read>(read: &mut T) -> Result<Option<Message>, Box<dyn std::error::Error>> {
    let mut size_buffer = [0u8; 4];
    match read.read_exact(&mut size_buffer) {
      Ok(_) => {}
      Err(ref e) if e.kind() == ErrorKind::BrokenPipe => { return Ok(None) }
      Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => { return Ok(None) }
      Err(e) => { return Err(e.into()) }
    }

    let size = u32::from_le_bytes(size_buffer);
    let mut data = vec![0u8; size as usize];
    match read.read_exact(&mut data) {
      Ok(_) => {}
      Err(ref e) if e.kind() == ErrorKind::BrokenPipe => { return Ok(None) }
      Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => { return Ok(None) }
      Err(e) => { return Err(e.into()) }
    }

    Ok(serde_cbor::from_slice(&data)?)
}

pub fn write<T: Write>(write: &mut T, msg: Message) -> Result<(), Box<dyn std::error::Error>> {
    let data = serde_cbor::to_vec(&msg)?;
    let size_buffer = (data.len() as u32).to_le_bytes();

    write.write(&size_buffer)?;
    write.write_all(&data)?;

    Ok(())
}

pub async fn async_read(read: &mut OwnedReadHalf) -> Result<Option<Message>, String> {

    let mut size_buffer = [0u8; 4];
    match read.read_exact(&mut size_buffer).await {
      Ok(_) => {}
      Err(ref e) if e.kind() == ErrorKind::BrokenPipe => { return Ok(None) }
      Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => { return Ok(None) }
      Err(e) => { return Err(e.to_string()) }
    }

    let size = u32::from_le_bytes(size_buffer);
    let mut data = vec![0u8; size as usize];
    match read.read_exact(&mut data).await {
      Ok(_) => {}
      Err(ref e) if e.kind() == ErrorKind::BrokenPipe => { return Ok(None) }
      Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => { return Ok(None) }
      Err(e) => { return Err(e.to_string()) }
    }

    let msg = serde_cbor::from_slice(&data).map_err(|e| e.to_string())?;
    Ok(Some(msg))
}

pub async fn async_write(write: &mut OwnedWriteHalf, msg: Message) -> Result<(), String> {
    let data = serde_cbor::to_vec(&msg).map_err(|e| e.to_string())?;
    let size_buffer = (data.len() as u32).to_le_bytes();

    write.write(&size_buffer).await.map_err(|e| e.to_string())?;
    write.write_all(&data).await.map_err(|e| e.to_string())?;

    Ok(())
}