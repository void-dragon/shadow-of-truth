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

  let shadow_shader = shader::new();
  let shadow_mvp = {
    let vertex_src = std::fs::read_to_string("assets/shaders/shadow.vertex.glsl").unwrap();
    let fragment_src = std::fs::read_to_string("assets/shaders/shadow.fragment.glsl").unwrap();
    let mut shadow_shader = shadow_shader.write().unwrap();
    shadow_shader.load(gl::VERTEX_SHADER, vertex_src).unwrap();
    shadow_shader.load(gl::FRAGMENT_SHADER, fragment_src).unwrap();
    shadow_shader.link().unwrap();

    shadow_shader.get_uniform_location("mvp".to_owned())
  };

  let draw_shader = shader::new();
  let (draw_attrib, draw_uni) = {
    let vertex_src = std::fs::read_to_string("assets/shaders/phong.vertex.glsl").unwrap();
    let fragment_src = std::fs::read_to_string("assets/shaders/phong.fragment.glsl").unwrap();
    let mut draw_shader = draw_shader.write().unwrap();
    draw_shader.load(gl::VERTEX_SHADER, vertex_src).unwrap();
    draw_shader.load(gl::FRAGMENT_SHADER, fragment_src).unwrap();
    draw_shader.link().unwrap();

    let d = DrawAttribLocations {
      material: MaterialLoc {
        ambient: 7,
        diffuse: 8,
        specular: 9,
        shininess: 10,
      },
      position: 0,
      normal: 1,
      texcoords: 2,
      t0: 3,
    };

    let u = DrawUniformLocations {
      light: LightLoc {
        position: draw_shader.get_uniform_location("light.position".to_owned()),
        ambient: draw_shader.get_uniform_location("light.ambient".to_owned()),
        diffuse: draw_shader.get_uniform_location("light.diffuse".to_owned()),
        specular: draw_shader.get_uniform_location("light.specular".to_owned()),
      },
      shadow_map: draw_shader.get_uniform_location("shadow_map".to_owned()),
      mvp: draw_shader.get_uniform_location("mvp".to_owned()),
      light_mvp: draw_shader.get_uniform_location("light_mvp".to_owned()),
    };

    (d, u)
  };

  Arc::new(RwLock::new(ImplScene {
    id: id,
    camera: camera,
    lights: vec![light],
    root: root,
    shadow_shader: shadow_shader,
    shadow_attrib_loc: ShadowAttribLocations {
      position: 0,
      t0: 2,
    },
    shadow_mvp_loc: shadow_mvp,
    draw_shader: draw_shader,
    draw_attrib_loc: draw_attrib,
    draw_unfirom_loc: draw_uni,
    models: HashMap::new(),
    drawables: HashMap::new(),
  }))
}

struct ShadowAttribLocations {
  position: u32,
  t0: u32,
}

struct LightLoc {
  position: i32,
  ambient: i32,
  diffuse: i32,
  specular: i32,
}

pub struct DrawUniformLocations {
  light: LightLoc,
  shadow_map: i32,
  mvp: i32,
  light_mvp: i32,
}

#[derive(Clone, Debug)]
pub struct MaterialLoc {
  pub ambient: u32,
  pub diffuse: u32,
  pub specular: u32,
  pub shininess: u32,
}

#[derive(Clone, Debug)]
pub struct DrawAttribLocations {
  pub material: MaterialLoc,
  pub position: u32,
  pub normal: u32,
  pub texcoords: u32,
  pub t0: u32,
}

pub struct ImplScene {
  pub id: String,
  pub camera: camera::Camera,
  pub lights: Vec<Light>,
  pub shadow_shader: Shader,
  shadow_attrib_loc: ShadowAttribLocations,
  shadow_mvp_loc: i32,
  pub draw_shader: Shader,
  draw_attrib_loc: DrawAttribLocations,
  draw_unfirom_loc: DrawUniformLocations,
  pub models: HashMap<String, Model>,
  pub drawables: HashMap<String, Drawable>,
  pub root: node::Node,
}

impl ImplScene {
  fn draw_shadows(&self) {
    unsafe {
      // gl::CullFace(gl::FRONT);
    }
    for light in &self.lights {
      unsafe {
        let light = light.read().unwrap();
        let shader = self.shadow_shader.read().unwrap();
        shader.bind();

        gl::Viewport(0, 0, light.shadow.width as _, light.shadow.height as _);
        gl::BindFramebuffer(gl::FRAMEBUFFER, light.shadow.fbo);
        gl::Clear(gl::DEPTH_BUFFER_BIT);

        gl::UniformMatrix4fv(self.shadow_mvp_loc, 1, gl::FALSE, light.mvp.as_ptr() as *const _);
      }

      self.draw_scene();
    }

    unsafe {
      gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
  }

  fn draw_objects(&self) {
    let cam = self.camera.read().unwrap();
    let mvp = cam.mvp.clone();
    unsafe {
      gl::Viewport(0, 0, cam.width as _, cam.height as _);
      gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
      // gl::CullFace(gl::BACK);
    }

    {
      let shader = self.draw_shader.read().unwrap();
      shader.bind();
    }

    unsafe {
      {
        let mvp = mvp.lock().unwrap();
        gl::UniformMatrix4fv(self.draw_unfirom_loc.mvp, 1, gl::FALSE, mvp.as_ptr() as *const _);
      }
      {
        let light = self.lights[0].read().unwrap();
        gl::UniformMatrix4fv(self.draw_unfirom_loc.light_mvp, 1, gl::FALSE, light.mvp.as_ptr() as *const _);
        // gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, light.shadow.depth_map);
        // gl::Uniform1i(self.draw_unfirom_loc.shadow_map, light.shadow.depth_map as _);
        // gl::Uniform1i(self.draw_unfirom_loc.shadow_map, 0);
      }

      {
        let light = &self.lights[0];
        let light = light.read().unwrap();
        gl::Uniform3fv(self.draw_unfirom_loc.light.position, 1, light.position.as_ptr());
        gl::Uniform3fv(self.draw_unfirom_loc.light.ambient, 1, light.ambient.as_ptr());
        gl::Uniform3fv(self.draw_unfirom_loc.light.diffuse, 1, light.diffuse.as_ptr());
        gl::Uniform3fv(self.draw_unfirom_loc.light.specular, 1, light.specular.as_ptr());
      }
    }

    self.draw_scene();
  }

  pub fn draw_scene(&self) {
    for drawable in &self.drawables {
      let drw = drawable.1.read().unwrap();

      unsafe { gl::BindVertexArray(drw.vao); }

      // index buffer has to be present, because it is not bound via context of vertex attribute diffenition
      drw.indices.bind();

      unsafe {
        // !! meditate about it !!
        // https://stackoverflow.com/questions/32447641/what-is-common-cause-of-range-out-of-bounds-of-buffer-in-webgl
        gl::DrawElementsInstanced(
          gl::TRIANGLES,
          drw.indices_count as _,
          gl::UNSIGNED_INT,
          std::ptr::null(),
          drw.references.len() as _,
        );
      }
    }
  }

  pub fn draw(&self) {
    let identity = matrix::new();
    self.root.read().unwrap().update_world_transform(&identity);

    for light in &self.lights {
      let mut light = light.write().unwrap();
      light.calculate();
    }

    self.camera.write().unwrap().calculate();

    for drawable in &self.drawables {
      drawable.1.write().unwrap().update_instance_matrices();
    }

    self.draw_shadows();
    self.draw_objects();
  }
}

pub type Scene = Arc<RwLock<ImplScene>>;

pub struct SceneUserData {
  pub scene: Scene,
}

use crate::methatron::drawable::DrawableUserData;

impl mlua::UserData for SceneUserData {
  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_method("create_model", |_, this, (name, filename): (String, String)| {
      log::debug!("create model {} {}", name, filename);

      let model = model::load(&filename).map_err(|e| error::to_lua_err(&e.to_string()))?;
      let mut scene = this.scene.write().map_err(|e| error::to_lua_err(&e.to_string()))?;
      scene.models.insert(name, model);
      Ok(())
    });

    methods.add_method("create_drawable", |_, this, (name, model): (String, String)| {
      let (model, locs) = {
        let scene = this.scene.read().map_err(|e| error::to_lua_err(&e.to_string()))?;
        let model = scene.models.get(&model).ok_or_else(|| error::to_lua_err("unknown models"))?.clone();
        (model, scene.draw_attrib_loc.clone())
      };
      let drawable = crate::methatron::drawable::new(locs, model);
      
      {
        let mut scene = this.scene.write().map_err(|e| error::to_lua_err(&e.to_string()))?;
        scene.drawables.insert(name, drawable.clone());
      }

      Ok(DrawableUserData(drawable))
    });

    methods.add_method("get_drawable", |_, this, name: String| {
      let scene = this.scene.read().map_err(|e| error::to_lua_err(&e.to_string()))?;
      let drawable = scene.drawables.get(&name).ok_or_else(|| error::to_lua_err("unknown shader"))?.clone();
      Ok(DrawableUserData(drawable))
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
