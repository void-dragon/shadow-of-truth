use std::io::{Read, Write};

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

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
  Login,
  Signed(Signed),
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

pub fn read<T: Read>(read: &mut T) -> Result<Message, Box<dyn std::error::Error>> {
    let mut size_buffer = [0u8; 4];
    read.read_exact(&mut size_buffer)?;

    let size = u32::from_le_bytes(size_buffer);
    let mut data = vec![0u8; size as usize];
    read.read_exact(&mut data)?;

    Ok(serde_cbor::from_slice(&data)?)
}

pub fn write<T: Write>(write: &mut T, msg: Message) -> Result<(), Box<dyn std::error::Error>> {
    let data = serde_cbor::to_vec(&msg).unwrap();
    let size_buffer = (data.len() as u32).to_le_bytes();

    write.write(&size_buffer)?;
    write.write(&data)?;

    Ok(())
}

pub async fn async_read(read: &mut OwnedReadHalf) -> Result<Message, Box<dyn std::error::Error>> {
    let mut size_buffer = [0u8; 4];
    read.read_exact(&mut size_buffer).await?;

    let size = u32::from_le_bytes(size_buffer);
    let mut data = vec![0u8; size as usize];
    read.read_exact(&mut data).await?;

    Ok(serde_cbor::from_slice(&data)?)
}

pub async fn async_write(write: &mut OwnedWriteHalf, msg: Message) -> Result<(), Box<dyn std::error::Error>> {
    let data = serde_cbor::to_vec(&msg).unwrap();
    let size_buffer = (data.len() as u32).to_le_bytes();

    write.write(&size_buffer).await?;
    write.write(&data).await?;

    Ok(())
}