use std::sync::atomic::{AtomicU64, Ordering};

static BEHAVIOR_ID: AtomicU64 = AtomicU64::new(0u64);

pub trait BehaviorExt: Send + Sync {
  fn id(&self) -> u64;

  fn on_update(&self);

  fn on_load(&self);
}

pub fn from_lua(path: &str) -> LuaBehavior {
  LuaBehavior {
    _id: BEHAVIOR_ID.fetch_add(1, Ordering::SeqCst),
  }
}

pub struct LuaBehavior {
  _id: u64,
}

impl BehaviorExt for LuaBehavior {
  fn id(&self) -> u64 { self._id }

  fn on_update(&self) {

  }

  fn on_load(&self) {

  }
}

pub fn load_module(lua: &mlua::Lua, ns: &mlua::Table) -> mlua::Result<()> {
  let module = lua.create_table()?;

  let from_file = lua.create_function(|_,_: ()| {
    Ok(())
  })?;
  module.set("from_file", from_file)?;

  ns.set("behavior", module)?;
  Ok(())
}