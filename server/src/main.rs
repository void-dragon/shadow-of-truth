use std::collections::{HashMap, HashSet};
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

#[derive(Clone)]
struct ServerContext {
    clients: RwClients,
    rooms: Arc<RwLock<HashMap<String, RwClients>>>,
    room_spawn_cache: Arc<RwLock<HashMap<String, HashMap<String, common::Message>>>>,
}

impl ServerContext {
    async fn fill_spawn_cache(&self, msg: &common::Message) {
        let mut cache = self.room_spawn_cache.write().await;
        if let common::Message::Spawn{id, scene, ..} = msg {
            let entry = cache.entry(scene.clone()).or_insert_with(|| HashMap::new());
            entry.insert(id.clone(), msg.clone());
        }
    }

    async fn clean_spawn_cache(&self, msg: &common::Message) {
        let mut cache = self.room_spawn_cache.write().await;
        if let common::Message::Destroy{id, scene} = msg {
            if let Some(entry) = cache.get_mut(scene) {
                entry.remove(id);
            }
        }
    }

    async fn send_spawn_cache(&self, scene: String, client: RwClient) {
        let cache = self.room_spawn_cache.read().await;
        if let Some(entry) = cache.get(&scene) {
            let c = client.read().await;
            for msg in entry.values() {
                c.tx.send(msg.clone()).await.unwrap();
            }
        }
    }

    async fn relay_message(&self, scene: &String, msg: &common::Message) {
        let rooms = self.rooms.read().await;
        if let Some(clients) = rooms.get(scene) {
            let clients = clients.read().await;

            for c in clients.values() {
                let client = c.read().await;
                client.tx.send(msg.clone()).await.unwrap();
            }
        }
    }

    async fn disconnect_client(&self, client: RwClient) {
        let (id, room) = {
            let mut c = client.write().await;
            c.state = client::ClientState::Disconnected;
            (c.id.clone(), c.room.clone())
        };
        
        {
            let mut clients = self.clients.write().await;
            clients.remove(&id);
        }

        {
            let rooms = self.rooms.read().await;
            if let Some(clients) = rooms.get(&room) {
                let mut clients = clients.write().await;
                clients.remove(&id);
            }
        }

        {
            let mut cache = self.room_spawn_cache.write().await;
            if let Some(r) = cache.get_mut(&room) {
                let client = client.read().await;
                for id in client.owned_spawns.iter() {
                    self.relay_message(&room, &common::Message::Destroy{scene: room.clone(), id: id.clone()}).await;
                    r.remove(id);
                }
            }
        }
    }
}

fn handle_stream(
    stream: TcpStream, 
    ctx: ServerContext,
) {
    let (mut read, mut write) = stream.into_split();
    let (tx, mut rx) = channel(20);
    let client = Arc::new(RwLock::new(client::Client {
        id: "".to_owned(),
        room: "".to_owned(),
        owned_spawns: HashSet::new(),
        state: client::ClientState::Greeting,
        tx: tx,
    }));

    tokio::spawn(async move {
        loop {
            match common::async_read(&mut read).await {
                Ok(msg) => {
                    match msg {
                        common::Message::Login{id} => {
                            {
                                let mut client = client.write().await;
                                client.id = id.clone();
                            }
                            let mut clients = ctx.clients.write().await;
                            clients.insert(id, client.clone());
                        }
                        common::Message::Join{scene} => {
                            let id = {
                                let mut c = client.write().await;
                                c.room = scene.clone();
                                c.id.clone()
                            };
                            let mut rooms = ctx.rooms.write().await;
                            let entry = rooms.entry(scene.clone()).or_insert_with(|| Arc::new(RwLock::new(HashMap::new())));
                            let mut room = entry.write().await; 
                            room.insert(id, client.clone());

                            ctx.send_spawn_cache(scene, client.clone()).await;
                        }
                        common::Message::Spawn{id, scene, drawable, behavior} => {
                            let spawn = common::Message::Spawn{id: id.clone(), scene: scene.clone(), drawable: drawable, behavior: behavior};
                            {
                                let mut client = client.write().await;
                                client.owned_spawns.insert(id);
                            }
                            ctx.fill_spawn_cache(&spawn).await;
                            ctx.relay_message(&scene, &spawn).await;
                        }
                        common::Message::Destroy{id, scene} => {
                            let destroy = common::Message::Destroy{id: id.clone(), scene: scene.clone()};
                            {
                                let mut client = client.write().await;
                                client.owned_spawns.remove(&id);
                            }
                            ctx.clean_spawn_cache(&destroy).await;
                            ctx.relay_message(&scene, &destroy).await;
                        }
                        common::Message::TransformUpdate{scene, id, t} => {
                            let msg = common::Message::TransformUpdate{scene: scene.clone(), id, t};
                            ctx.relay_message(&scene, &msg).await;
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
        ctx.disconnect_client(client).await;
    });

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = common::async_write(&mut write, msg).await {
                log::error!("writer {}", e.to_string());
                break;
            }
        }
    });
}



async fn listen(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("127.0.0.1:{}", config.port);
    let filename = std::path::Path::new(&config.private_key);
    // let private_key = optain_private_key(&filename);
    let listener = TcpListener::bind(addr).await?;
    let ctx = ServerContext {
        clients: Arc::new(RwLock::new(HashMap::new())),
        rooms: Arc::new(RwLock::new(HashMap::new())),
        room_spawn_cache: Arc::new(RwLock::new(HashMap::new())),
    };

    log::info!("listen on {}", listener.local_addr()?);
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                log::info!("connection from {:?}", addr);
                handle_stream(stream, ctx.clone());
            }
            Err(e) => log::error!("listener: {}", e.to_string())
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
