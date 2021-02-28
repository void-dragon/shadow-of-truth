use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::Arc;

use env_logger::Env;
use tokio::{
    net::{
        TcpStream,
        TcpListener,
    },
    sync::{
        mpsc::channel,
        RwLock,
    }
};


mod config;
mod client;

use config::Config;

use shadow_of_truth_common as common;

type RwClient = Arc<RwLock<client::Client>>;
type RwClients = Arc<RwLock<HashMap<String, RwClient>>>;

fn handle_stream(
    stream: TcpStream, 
    clients: RwClients,
    rooms: Arc<RwLock<HashMap<String, RwClients>>>,
) {
    let (mut read, mut write) = stream.into_split();
    let (tx, mut rx) = channel(20);
    let client = Arc::new(RwLock::new(client::Client {
        id: "".to_owned(),
        state: client::ClientState::Greeting,
        tx: tx,
    }));

    tokio::spawn(async move {
        loop {
            match common::async_read(&mut read).await {
                Ok(msg) => {
                    log::debug!("{:?}", msg);
                    match msg {
                        common::Message::Login{id} => {
                            {
                                let mut client = client.write().await;
                                client.id = id.clone();
                            }
                            let mut clients = clients.write().await;
                            clients.insert(id, client.clone());
                        }
                        common::Message::Join{scene} => {
                            let mut rooms = rooms.write().await;
                            let entry = rooms.entry(scene).or_insert_with(|| Arc::new(RwLock::new(HashMap::new())));
                            let id = {
                                let c = client.read().await;
                                c.id.clone()
                            };
                            let mut room = entry.write().await; 
                            room.insert(id, client.clone());
                        }
                        common::Message::Spawn{id, scene, drawable, behavior} => {
                            let rooms = rooms.read().await;
                            if let Some(clients) = rooms.get(&scene) {
                                let clients = clients.read().await;
                                let spawn = common::Message::Spawn{id: id, scene: scene, drawable: drawable, behavior: behavior};
                                for c in clients.values() {
                                    let client = c.read().await;
                                    client.tx.send(spawn.clone()).await;
                                }
                            }
                        }
                        common::Message::TransformUpdate{scene, id, t} => {
                            let rooms = rooms.read().await;
                            if let Some(recipents) = rooms.get(&scene) {
                                let recipents = recipents.read().await;
                                let msg = common::Message::TransformUpdate{scene, id, t};
                                for client in recipents.values() {
                                    let client = client.read().await;
                                    client.tx.send(msg.clone()).await;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    log::error!("read {}", e);
                    break;
                }
            }
        }
        let id = {
            let mut c = client.write().await;
            c.state = client::ClientState::Disconnected;
            c.id.clone()
        };
        let mut clients = clients.write().await;
        clients.remove(&id);
    });

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = common::async_write(&mut write, msg).await {
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
    let rooms = Arc::new(RwLock::new(HashMap::new()));

    log::info!("listen on {}", listener.local_addr()?);
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                log::info!("connection from {:?}", addr);
                handle_stream(stream, clients.clone(), rooms.clone());
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
