use std::sync::{Arc, RwLock};

use crate::methatron::{math::matrix, node};

pub fn new(width: usize, height: usize) -> Camera {
  let p = matrix::new();
  matrix::perspective(&mut p.lock().unwrap(), 45.0, width as f32 / height as f32, 0.1, 1000.0);

  Arc::new(RwLock::new(ImplCamera {
    node: node::new(),
    perspective: p,
    mvp: matrix::new(),
  }))
}

pub type Camera = Arc<RwLock<ImplCamera>>;

pub struct ImplCamera {
  pub node: node::Node,
  pub perspective: matrix::Matrix,
  pub mvp: matrix::Matrix,
}

impl ImplCamera {
  pub fn calculate(&self) {
    let mut inverse = [0.0; 16];

    let mut mvp = self.mvp.lock().unwrap();
    matrix::identity(&mut mvp);
    matrix::mul_assign(&mut mvp, &self.perspective.lock().unwrap());
    let node = self.node.read().unwrap();
    matrix::inverse(&node.world_transform.lock().unwrap(), &mut inverse);
    matrix::mul_assign(&mut mvp, &inverse);
  }

  pub fn resize(&self, width: usize, height: usize) {
    matrix::perspective(&mut self.perspective.lock().unwrap(), 45.0, width as f32 / height as f32, 0.1, 1000.0);
  }
}

pub struct CameraUserData {
  pub camera: Camera
}

impl mlua::UserData for CameraUserData {
  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_method("get_node", |_, this, ()| {
      use crate::methatron::node::NodeUserData;
      let node = this.camera.read().unwrap().node.clone();
      Ok(NodeUserData { node: node })
    });
  }
}
