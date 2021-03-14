use std::sync::{Arc, RwLock};

use crate::methatron::{
  math::matrix,
  node::{self, Node},
  shadow::{self, Shadow},
};

pub struct ImplLight {
  pub node: Node,
  pub target: Option<Node>,
  pub shadow: Shadow,
  pub mvp: [f32; 16],
  pub position: [f32; 3],
  pub ambient: [f32; 3],
  pub diffuse: [f32; 3],
  pub specular: [f32; 3],
}

impl ImplLight {
  pub fn calculate(&mut self) {
    let projection = matrix::ortho(-20.0, 20.0, -20.0, 20.0, 0.01, 20.5);
    let mut origin = self.node.read().unwrap().world_transform.lock().unwrap().clone();
    let mut inverse = [0.0; 16];
    matrix::identity(&mut self.mvp);
    matrix::identity(&mut inverse);

    let at = if let Some(ref target) = self.target {
      let t = target.read().unwrap();
      let m = t.world_transform.lock().unwrap();

      [m[12], m[13], m[14]]
    }
    else {
      [0.0, 0.0, 0.0]
    };

    matrix::look_at(&mut origin, &at, &[0.0, 1.0, 0.0]);

    matrix::inverse(&origin, &mut inverse);
    matrix::mul_assign(&mut self.mvp, &projection);
    matrix::mul_assign(&mut self.mvp, &inverse);
    self.position[0] = origin[12];
    self.position[1] = origin[13];
    self.position[2] = origin[14];
  }
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

    methods.add_method("get_target", |l, this, ()| {
      use crate::methatron::node::NodeUserData;
      use mlua::ToLua;

      let node = this.0.read().unwrap();

      if let Some(ref target) = node.target {
        Ok(NodeUserData{node: target.clone()}.to_lua(&l).unwrap())
      }
      else {
        Ok(mlua::Value::Nil)
      }
    });

    methods.add_method("set_target", |_, this, node: mlua::AnyUserData| {
      use crate::methatron::node::NodeUserData;
      let target = node.borrow::<NodeUserData>().unwrap();
      this.0.write().unwrap().target = Some(target.node.clone());
      Ok(())
    });
  }
}

pub fn new() -> Light {
  let light = ImplLight {
    node: node::new(),
    target: None,
    shadow: shadow::new(2048, 2048),
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