use std::collections::{HashMap, HashSet};
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
  owned: Arc<RwLock<HashSet<String>>>,
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
            log::debug!("{:?}", msg);
            match msg {
              common::Message::Spawn{id, scene, drawable, behavior} => {
                // crate::methatron::drawable::new(shader: Shader, model: Model)
                let ctx = ctx.read().unwrap();

                if let Some(ref scene) = ctx.scene {
                  let mut scene = scene.write().unwrap();
                  let node = crate::methatron::node::new();
                  let drawble = scene.drawables.get(&drawable).unwrap().clone();
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
                      owned.insert(id.clone());
                      let mut opt_node = pair.0.lock().unwrap();
                      *opt_node = Some(node.clone());
                      pair.1.notify_one();
                    }
                  }
                }
              }
              common::Message::Destroy{id} => {}
              common::Message::TransformUpdate{id, scene, t} => {

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

    methods.add_method("update", |_, this, (scene, node): (String, mlua::AnyUserData)| {
      let n = node.borrow::<crate::methatron::node::NodeUserData>().unwrap();
      let node = n.node.read().unwrap();
      this.send(common::Message::TransformUpdate {
        scene: scene,
        id: node.network_id.clone(),
        t: node.transform.lock().unwrap().clone(),
      });

      Ok(())
    });
  }
}

pub fn new() -> Network {
  let net = Network {
    id: nanoid::nanoid!(32),
    writer: Arc::new(Mutex::new(None)),
    synced_nodes: Arc::new(RwLock::new(HashMap::new())),
    owned: Arc::new(RwLock::new(HashSet::new())),
    waiting: Arc::new(RwLock::new(HashMap::new())),
  };

  net.establish_connection();

  net
}