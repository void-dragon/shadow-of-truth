use env_logger::Env;
use tokio::{
    net::{
        TcpStream,
        TcpListener,
    },
    sync::mpsc::channel,
};
// use openssl::rsa::{Rsa, Padding};
// use openssl::pkey::{PKey, Public, Private};
use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::{Read, Write};


mod config;
mod client;
mod message;

use config::Config;


fn handle_stream(stream: TcpStream) {
    let (mut read, mut write) = stream.into_split();
    let (tx, mut rx) = channel(20);

    tokio::spawn(async move {
        loop {
            match message::read(&mut read).await {
                Ok(msg) => {
                    log::info!("{:?}", msg);
                }
                Err(e) => {
                    log::error!("{}", e.to_string());
                    break;
                }
            }
        }
    });

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = message::write(&mut write, msg).await {
                log::error!("{}", e.to_string());
            }
        }
    });
}

// pub fn optain_private_key(path: &std::path::Path) -> PKey<Private> {
//   if path.exists() {
//     load_key(path)
//   }
//   else {
//     generate_key(path)
//   }
// }
// 
// fn generate_key(path: &std::path::Path) -> PKey<Private> {
//     log::info!("generate private key");
//     let key = Rsa::generate(4096).unwrap();
//     let pkey = PKey::from_rsa(key).unwrap();
// 
//     let data = pkey.private_key_to_pem_pkcs8().unwrap();
//     let mut file = File::create(path).unwrap();
//     file.write(&data).unwrap();
// 
//     pkey
// }
// 
// fn load_key(path: &std::path::Path) -> PKey<Private> {
//     log::info!("load private key");
//     let mut file = File::open(path).unwrap();
//     let mut buffer = Vec::new();
// 
//     file.read_to_end(&mut buffer).unwrap();
// 
//     PKey::private_key_from_pem(&buffer).unwrap()
// }


async fn listen(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("127.0.0.1:{}", config.port);
    let filename = std::path::Path::new(&config.private_key);
    // let private_key = optain_private_key(&filename);
    let listener = TcpListener::bind(addr).await?;

    log::info!("listen on port {}", listener.local_addr()?);
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                log::info!("connection from {:?}", addr);
                handle_stream(stream);
            }
            Err(e) => log::error!("{:?}", e)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = Env::default().default_filter_or("debug");
    env_logger::Builder::from_env(env).init();

    match config::load() {
        Ok(config) => {
            tokio::spawn(async move {
                if let Err(e) = listen(config).await {
                    log::error!("{}", e.to_string());
                }
            });

            tokio::signal::ctrl_c().await?;
        }
        Err(e) => log::error!("{}", e.to_string()),
    }


    Ok(())
}
