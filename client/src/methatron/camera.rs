use std::sync::{Arc, RwLock};

use crate::methatron::{math::matrix, node};

pub fn new(width: usize, height: usize) -> Camera {
  let p = matrix::new();
  matrix::perspective(&p, 45.0, width as f32 / height as f32, 0.1, 1000.0);

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
    let inverse = matrix::new();

    matrix::identity(&self.mvp);
    matrix::mul_assign(&self.mvp, &self.perspective);
    let node = self.node.read().unwrap();
    matrix::inverse(&node.world_transform, &inverse);
    matrix::mul_assign(&self.mvp, &inverse);
  }

  pub fn resize(&self, width: usize, height: usize) {
    matrix::perspective(&self.perspective, 45.0, width as f32 / height as f32, 0.1, 1000.0);
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
