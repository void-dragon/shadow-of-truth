use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::methatron::{camera, drawable::Drawable, node, math::matrix};
use gl::types::GLuint;

pub fn new() -> Scene {
  let root = node::new();
  let camera = camera::new(1024, 780);

  root.write().unwrap().add_child(camera.read().unwrap().node.clone());

  Arc::new(RwLock::new(ImplScene {
    camera: camera,
    root: root,
    drawables: HashMap::new(),
  }))
}

pub struct ImplScene {
  pub camera: camera::Camera,
  pub drawables: HashMap<GLuint, Drawable>,
  pub root: node::Node,
}

impl ImplScene {
  pub fn spawn(&self, drawable: Drawable) -> node::Node {
    let node = node::new();

    let id = node.read().unwrap().id();
    {
      let mut d = drawable.write().unwrap();
      d.references.insert(id, node.clone());
    }

    node.write().unwrap().drawable = Some(drawable);

    node
  }

  pub fn draw(&self) {
    let identity = matrix::new();
    self.root.read().unwrap().update_world_transform(&identity);

    let mvp = {
      let cam = self.camera.write().unwrap();
      cam.calculate();

      cam.mvp.clone()
    };

    for drawable in &self.drawables {
      drawable.1.write().unwrap().draw(&mvp);
    }
  }
}

pub type Scene = Arc<RwLock<ImplScene>>;

pub struct SceneUserData {
  pub scene: Scene,
}

use crate::methatron::drawable::DrawableUserData;

impl mlua::UserData for SceneUserData {
  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_method("add_drawable", |_, this, drawable: mlua::AnyUserData| {
      let mut scene = this.scene.write().unwrap();
      let bd = drawable.borrow::<DrawableUserData>().unwrap();
      let id = bd.drawable.read().unwrap().id();
      scene.drawables.insert(id, bd.drawable.clone());
      Ok(())
    });

    methods.add_method("get_root", |_, this, _: ()| {
      use crate::methatron::node::NodeUserData;

      let scene = this.scene.read().unwrap();
      let root = scene.root.clone();
      Ok(NodeUserData { node: root })
    });

    methods.add_method("get_camera", |_, this, _: ()| {
      use crate::methatron::camera::CameraUserData;

      let scene = this.scene.read().unwrap();
      let camera = scene.camera.clone();
      Ok(CameraUserData { camera: camera })
    });
  }
}

pub fn load_module(lua: &mlua::Lua, ns: &mlua::Table) -> Result<(), mlua::Error> {
  let module = lua.create_table()?;

  let lua_new = lua.create_function(|_, _: ()| {
    Ok(SceneUserData { scene: new() })
  })?;
  module.set("new", lua_new)?;

  ns.set("scene", module)?;

  Ok(())
}
