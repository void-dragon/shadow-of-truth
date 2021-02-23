use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use env_logger::Env;
use tokio::{
    net::{
        TcpStream,
        TcpListener,
    },
    sync::mpsc::channel,
};


mod config;
mod client;

use config::Config;

use shadow_of_truth_common as message;

fn handle_stream(stream: TcpStream, clients: Arc<RwLock<HashMap<String, Arc<RwLock<client::Client>>>>>) {
    let (mut read, mut write) = stream.into_split();
    let (tx, mut rx) = channel(20);
    let client = Arc::new(RwLock::new(client::Client {
        id: "".to_owned(),
        state: client::ClientState::Greeting,
        tx: tx,
    }));

    tokio::spawn(async move {
        loop {
            match message::async_read(&mut read).await {
                Ok(msg) => {
                    log::info!("{:?}", msg);
                }
                Err(e) => {
                    let mut c = client.write().unwrap();
                    c.state = client::ClientState::Disconnected;
                    log::error!("read {}", e.to_string());
                    break;
                }
            }
        }
    });

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = message::async_write(&mut write, msg).await {
                log::error!("writer {}", e.to_string());
            }
        }
    });
}



async fn listen(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("127.0.0.1:{}", config.port);
    let filename = std::path::Path::new(&config.private_key);
    // let private_key = optain_private_key(&filename);
    let listener = TcpListener::bind(addr).await?;
    let clients = Arc::new(RwLock::new(HashMap::new()));

    log::info!("listen on {}", listener.local_addr()?);
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                log::info!("connection from {:?}", addr);
                handle_stream(stream, clients.clone());
            }
            Err(e) => log::error!("{}", e.to_string())
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
