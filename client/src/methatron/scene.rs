use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::methatron::{
  error,
  camera,
  drawable::Drawable,
  node,
  model::{self, Model},
  math::matrix,
  light::{self, Light},
  shader::{self, Shader}
};

pub fn new(id: String) -> Scene {
  let root = node::new();
  let camera = camera::new(1024, 780);
  let light = light::new();

  {
    let mut root = root.write().unwrap();
    root.add_child(camera.read().unwrap().node.clone());
    root.add_child(light.read().unwrap().node.clone());
  }

  Arc::new(RwLock::new(ImplScene {
    id: id,
    camera: camera,
    lights: vec![light],
    root: root,
    models: HashMap::new(),
    shaders: HashMap::new(),
    drawables: HashMap::new(),
  }))
}

pub struct ImplScene {
  pub id: String,
  pub camera: camera::Camera,
  pub lights: Vec<Light>,
  pub models: HashMap<String, Model>,
  pub shaders: HashMap<String, Shader>,
  pub drawables: HashMap<String, Drawable>,
  pub root: node::Node,
}

impl ImplScene {
  pub fn draw(&self) {
    let identity = matrix::new();
    self.root.read().unwrap().update_world_transform(&identity);

    for light in &self.lights {
      let mut l = light.write().unwrap();
      let n = l.node.clone();
      let n = n.read().unwrap();
      let w = n.world_transform.lock().unwrap();
      l.position[0] = w[12];
      l.position[1] = w[13];
      l.position[2] = w[14];
    }

    let mvp = {
      let cam = self.camera.write().unwrap();
      cam.calculate();

      cam.mvp.clone()
    };

    for drawable in &self.drawables {
      drawable.1.write().unwrap().draw(&mvp, &self.lights);
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
    methods.add_method("create_shader", |_, this, (id, vertex, fragment): (String, String, String)| {

      let shader = shader::new();
      {
        let vertex_src = std::fs::read_to_string(vertex)?;
        let fragment_src = std::fs::read_to_string(fragment)?;
        let mut shader = shader.write().map_err(|e| error::to_lua_err(&e.to_string()))?;
        shader.load(gl::VERTEX_SHADER, vertex_src).map_err(|e| error::to_lua_err(&e))?;
        shader.load(gl::FRAGMENT_SHADER, fragment_src).map_err(|e| error::to_lua_err(&e))?;
        shader.link().map_err(|e| error::to_lua_err(&e))?;
      }

      let mut scene = this.scene.write().map_err(|e| error::to_lua_err(&e.to_string()))?;
      scene.shaders.insert(id, shader);

      Ok(())
    });

    methods.add_method("create_model", |_, this, (name, filename): (String, String)| {
      let model = model::load(&filename).map_err(|e| error::to_lua_err(&e.to_string()))?;
      let mut scene = this.scene.write().map_err(|e| error::to_lua_err(&e.to_string()))?;
      scene.models.insert(name, model);
      Ok(())
    });

    methods.add_method("create_drawable", |_, this, (name, shader, model): (String, String, String)| {
      let (shader, model) = {
        let scene = this.scene.read().map_err(|e| error::to_lua_err(&e.to_string()))?;
        let shader = scene.shaders.get(&shader).ok_or_else(|| error::to_lua_err("unknown shader"))?.clone();
        let model = scene.models.get(&model).ok_or_else(|| error::to_lua_err("unknown models"))?.clone();
        (shader, model)
      };
      let drawable = crate::methatron::drawable::new(shader, model);
      
      {
        let mut scene = this.scene.write().map_err(|e| error::to_lua_err(&e.to_string()))?;
        scene.drawables.insert(name, drawable.clone());
      }

      Ok(DrawableUserData {drawable: drawable})
    });

    methods.add_method("get_drawable", |_, this, name: String| {
      let scene = this.scene.read().map_err(|e| error::to_lua_err(&e.to_string()))?;
      let drawable = scene.drawables.get(&name).ok_or_else(|| error::to_lua_err("unknown shader"))?.clone();
      Ok(DrawableUserData {drawable: drawable})
    });

    methods.add_method("id", |_, this, _: ()| {
      let scene = this.scene.read().map_err(|e| error::to_lua_err(&e.to_string()))?;
      let id = scene.id.clone();
      Ok(id)
    });

    methods.add_method("get_root", |_, this, _: ()| {
      use crate::methatron::node::NodeUserData;

      let scene = this.scene.read().map_err(|e| error::to_lua_err(&e.to_string()))?;
      let root = scene.root.clone();
      Ok(NodeUserData { node: root })
    });

    methods.add_method("get_camera", |_, this, _: ()| {
      use crate::methatron::camera::CameraUserData;

      let scene = this.scene.read().map_err(|e| error::to_lua_err(&e.to_string()))?;
      let camera = scene.camera.clone();
      Ok(CameraUserData { camera: camera })
    });

    methods.add_method("get_lights", |_, this, _: ()| {
      use crate::methatron::light::LightUserData;

      let scene = this.scene.read().map_err(|e| error::to_lua_err(&e.to_string()))?;
      let lights: Vec<LightUserData> = scene.lights.iter().map(|l| LightUserData(l.clone())).collect();
      Ok(lights)
    });
  }
}

pub fn load_module(lua: &mlua::Lua, ns: &mlua::Table) -> Result<(), mlua::Error> {
  let module = lua.create_table()?;

  let lua_new = lua.create_function(|_, id: String| {
    Ok(SceneUserData { scene: new(id) })
  })?;
  module.set("new", lua_new)?;

  ns.set("scene", module)?;

  Ok(())
}
