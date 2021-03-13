use std::sync::{Arc, RwLock};

use crate::methatron::{
  node::{self, Node},
  shadow::{self, Shadow},
};

pub struct ImplLight {
  pub node: Node,
  pub shadow: Shadow,
  pub mvp: [f32; 16],
  pub position: [f32; 3],
  pub ambient: [f32; 3],
  pub diffuse: [f32; 3],
  pub specular: [f32; 3],
}

pub type Light = Arc<RwLock<ImplLight>>;

pub struct LightUserData(pub Light);

impl mlua::UserData for LightUserData {
  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_method("get_node", |_, this, ()| {
      use crate::methatron::node::NodeUserData;
      let node = this.0.read().unwrap().node.clone();
      Ok(NodeUserData { node: node })
    });
  }
}

pub fn new() -> Light {
  let light = ImplLight {
    node: node::new(),
    shadow: shadow::new(1024, 1024),
    mvp: [0.0; 16],
    position: [0.0, 0.0, 0.0],
    ambient: [1.0, 1.0, 1.0],
    diffuse: [1.0, 1.0, 1.0],
    specular: [1.0, 1.0, 1.0],
  };
  Arc::new(RwLock::new(light))
}

pub fn load_module(lua: &mlua::Lua, ns: &mlua::Table) -> mlua::Result<()> {
  let module = lua.create_table()?;

  let new_node = lua.create_function(|_,_: ()| {
    Ok(LightUserData(new()))
  })?;
  module.set("new", new_node)?;

  ns.set("light", module)?;

  Ok(())
}