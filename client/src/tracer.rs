
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

pub struct ImplTracer {
  starter: BTreeMap<String, SystemTime>,
}

pub type Tracer = Arc<RwLock<ImplTracer>>;

pub fn new() -> TracerUserData {
  TracerUserData {
    tracer: Arc::new(RwLock::new(ImplTracer {
      starter: BTreeMap::new(),
    })),
  }
}

pub struct TracerUserData {
  tracer: Tracer,
}

impl mlua::UserData for TracerUserData {
  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_method("start", |_, this, name: String| {
      let mut tracer = this.tracer.write().unwrap();
      tracer.starter.insert(name, SystemTime::now());
      Ok(())
    });

    methods.add_method("stop", |_, this, name: String| {
      let mut tracer = this.tracer.write().unwrap();
      {
        let start = tracer.starter.get(&name).unwrap();
        let duration = start.elapsed().unwrap();
        println!("trace [{}] {}.{:09}", name, duration.as_secs(), duration.subsec_nanos());
      }
      tracer.starter.remove(&name);
      Ok(())
    });
  }
}
