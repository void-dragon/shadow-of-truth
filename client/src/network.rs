use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{Arc, Mutex, RwLock, Condvar};
use std::time::Duration;

use shadow_of_truth_common as common;
use crate::{
  events,
  methatron::node::{
    Node,
    NodeUserData,
  }
};

#[derive(Clone)]
pub struct Network {
  id: String,
  writer: Arc<Mutex<Option<TcpStream>>>,
  owned: Arc<RwLock<HashMap<String, (String, Node)>>>,
  synced_nodes: Arc<RwLock<HashMap<String, Node>>>,
  waiting: Arc<RwLock<HashMap<String, Arc<(Mutex<Option<Node>>, Condvar)>>>>,
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

        std::thread::sleep(Duration::from_secs(count * 10));
      }
    });
  }

  pub fn try_connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = TcpStream::connect("127.0.0.1:3000")?;
    {
      let mut w = self.writer.lock().unwrap();
      *w = Some(reader.try_clone()?);
    }

    self.send(common::Message::Login{id: self.id.clone()});

    let ep = events::get();
    ep.sender.send(events::Events::Connected)?;


    let network = self.clone();
    std::thread::spawn(move || {
      let ctx = crate::context::get();

      loop {
        match common::read(&mut reader) {
          Ok(msg) => {
            match &msg {
              &common::Message::TransformUpdate{..} => {}
              m @ _ => {log::debug!("{:?}", m)}
            }

            match msg {
              common::Message::Spawn{id, scene, drawable, behavior} => {
                let ctx = ctx.read().unwrap();

                if let Some(ref sc) = ctx.scene {
                  let sc = sc.read().unwrap();
                  let node = crate::methatron::node::new();
                  let drawble = sc.drawables.get(&drawable).unwrap().clone();
                  {
                    let mut n = node.write().unwrap();
                    n.network_id = id.clone();
                    n.set_drawable(drawble);
                  }
                  {
                    let mut nodes = network.synced_nodes.write().unwrap();
                    nodes.insert(id.clone(), node.clone());
                  }
                  {
                    let mut waiters = network.waiting.write().unwrap();
                    if let Some(pair) = waiters.remove(&id) {
                      let mut owned = network.owned.write().unwrap();
                      owned.insert(id.clone(), (scene, node.clone()));
                      let mut opt_node = pair.0.lock().unwrap();
                      *opt_node = Some(node.clone());
                      pair.1.notify_one();
                    }
                  }
                  sc.root.write().unwrap().add_child(node);
                }
              }
              common::Message::Destroy{id, scene} => {}
              common::Message::TransformUpdate{id, scene, t} => {
                let nodes = network.synced_nodes.read().unwrap();
                if let Some(node) = nodes.get(&id) {
                  // log::debug!("sync {} {:?}", id, t);
                  let node = node.read().unwrap();
                  let mut m = node.transform.lock().unwrap();

                  for i in 0..16 {
                    m[i] = t[i];
                  }
                }
              }
              _ => {}
            }
          }
          Err(e) => {
            log::error!("read {}", e.to_string());
            break;
          }
        }
      }
    });

    let network = self.clone();
    std::thread::spawn(move || {
      loop {
        {
          let mut writer = network.writer.lock().unwrap();
          if let Some(ref mut writer) = *writer {

            let owned = network.owned.read().unwrap();
            for (scene, node) in owned.values() {
              let node = node.read().unwrap();
              let msg = common::Message::TransformUpdate {
                id: node.network_id.clone(),
                t: node.transform.lock().unwrap().clone(),
                scene: scene.clone(),
              };

              if let Err(e) = common::write(writer, msg) {
                log::error!("transform update {}", e.to_string());
              }
            }
          }
        }

        std::thread::sleep(std::time::Duration::from_millis(50));
      }
    });

    Ok(())
  }

  pub fn send(&self, msg: common::Message) {
    let mut writer = self.writer.lock().unwrap();
    if let Some(ref mut writer) = *writer {
      if let Err(e) = common::write(writer, msg) {
        log::error!("write {}", e.to_string());
      }
    }
  }
}

impl mlua::UserData for Network {
  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_method("spawn", |_, this, (scene, drawable, behave): (String, String, String)| {
      let id = nanoid::nanoid!(32);
      let pair = {
        let mut waiters = this.waiting.write().unwrap();
        let pair = Arc::new((Mutex::new(None), Condvar::new()));
        waiters.insert(id.clone(), pair.clone());
        pair
      };

      this.send(common::Message::Spawn {
        id: id.clone(),
        scene: scene,
        drawable: drawable,
        behavior: behave,
      });

      let (lock, cvar) = &*pair;
      let mut opt_node = lock.lock().unwrap();
      while opt_node.is_none() {
        opt_node = cvar.wait(opt_node).unwrap();
      }

      {
        let mut waiters = this.waiting.write().unwrap();
        waiters.remove(&id);
      }

      Ok(NodeUserData { node: opt_node.clone().unwrap() })
    });

    methods.add_method("join", |_, this, scene: String| {
      this.send(common::Message::Join { scene: scene });

      Ok(())
    });
  }
}

pub fn new() -> Network {
  let net = Network {
    id: nanoid::nanoid!(32),
    writer: Arc::new(Mutex::new(None)),
    synced_nodes: Arc::new(RwLock::new(HashMap::new())),
    owned: Arc::new(RwLock::new(HashMap::new())),
    waiting: Arc::new(RwLock::new(HashMap::new())),
  };

  net.establish_connection();

  net
}