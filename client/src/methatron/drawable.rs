use gl::types::{GLint, GLuint};

use crate::methatron::{
  pump, 
  material,
  math::matrix::Matrix, 
  model::Model, 
  node::Node, 
  shader::Shader, 
  vbo::VBO
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub fn new(shader: Shader, model: Model) -> Drawable {
  let pump = pump::get();

  let (vao, indices, vertices, normals, transforms, materials) = {
    let shader = shader.clone();
    let model = model.clone();
    pump.exec(move || {

      let mut vao = 0;

      unsafe {
        gl::CreateVertexArrays(1, &mut vao as *mut _);
        gl::BindVertexArray(vao);
      }

      // !! NOTE !!
      // The order of VBO creation is important!
      // The bound VBO will be associated with the vertex attribute array configuration.

      let float_size = 4; // float32 are 4 bytes :)
      let model = model.read().unwrap();

      let indices = VBO::new(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW, &model.indices);
      let vertices = VBO::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW, &model.vertices);

      unsafe {
        let shader = shader.read().unwrap();
        gl::EnableVertexAttribArray(shader.position);
        gl::VertexAttribPointer(shader.position as _, 3, gl::FLOAT, gl::FALSE, float_size * 3, std::ptr::null());
      }

      let normals = VBO::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW, &model.normals);

      unsafe {
        let shader = shader.read().unwrap();
        gl::EnableVertexAttribArray(shader.normal);
        gl::VertexAttribPointer(shader.normal, 3, gl::FLOAT, gl::FALSE, float_size * 3, std::ptr::null());
      }

      // let textures = VBO::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW, &buffer);

      let transform_buffer: Vec<f32> = vec![0.0; 16];
      let transforms = VBO::new(gl::ARRAY_BUFFER, gl::DYNAMIC_DRAW, &transform_buffer);

      let matrix_offset = float_size * 16;

      unsafe {
        let shader = shader.read().unwrap();
        gl::EnableVertexAttribArray(shader.t0 + 0);
        gl::EnableVertexAttribArray(shader.t0 + 1);
        gl::EnableVertexAttribArray(shader.t0 + 2);
        gl::EnableVertexAttribArray(shader.t0 + 3);
        gl::VertexAttribPointer(shader.t0 + 0, 4, gl::FLOAT, gl::FALSE, matrix_offset, 0 as *const _);
        gl::VertexAttribPointer(shader.t0 + 1, 4, gl::FLOAT, gl::FALSE, matrix_offset, (float_size * 4) as *const _);
        gl::VertexAttribPointer(shader.t0 + 2, 4, gl::FLOAT, gl::FALSE, matrix_offset, (float_size * 8) as *const _);
        gl::VertexAttribPointer(shader.t0 + 3, 4, gl::FLOAT, gl::FALSE, matrix_offset, (float_size * 12) as *const _);
        gl::VertexAttribDivisor(shader.t0 + 0, 1);
        gl::VertexAttribDivisor(shader.t0 + 1, 1);
        gl::VertexAttribDivisor(shader.t0 + 2, 1);
        gl::VertexAttribDivisor(shader.t0 + 3, 1);
      }

      let material_buffer: Vec<f32> = vec![1.0; 10];
      let materials = VBO::new(gl::ARRAY_BUFFER, gl::DYNAMIC_DRAW, &material_buffer);
      let material_offset = float_size * 10;
      unsafe {
        let shader = shader.read().unwrap();
        gl::EnableVertexAttribArray(shader.material.ambient);
        gl::EnableVertexAttribArray(shader.material.diffuse);
        gl::EnableVertexAttribArray(shader.material.specular);
        gl::EnableVertexAttribArray(shader.material.shininess);
        gl::VertexAttribPointer(shader.material.ambient, 3, gl::FLOAT, gl::FALSE, material_offset, 0 as *const _);
        gl::VertexAttribPointer(shader.material.diffuse, 3, gl::FLOAT, gl::FALSE, material_offset, (float_size * 3) as *const _);
        gl::VertexAttribPointer(shader.material.specular, 3, gl::FLOAT, gl::FALSE, material_offset, (float_size * 6) as *const _);
        gl::VertexAttribPointer(shader.material.shininess, 1, gl::FLOAT, gl::FALSE, material_offset, (float_size * 9) as *const _);
        gl::VertexAttribDivisor(shader.material.ambient, 1);
        gl::VertexAttribDivisor(shader.material.diffuse, 1);
        gl::VertexAttribDivisor(shader.material.specular, 1);
        gl::VertexAttribDivisor(shader.material.shininess, 1);
      }

      (vao, indices, vertices, normals, transforms, materials)
    })
  };
  log::debug!("created drawable {}", vao);

  Arc::new(RwLock::new(ImplDrawable {
    vao: vao,
    indices: indices,
    vertices: vertices,
    normals: normals,
    // textures: textures,
    transforms: transforms,
    transform_buffer: vec![0.0; 16],
    references: HashMap::new(),
    shader: shader,
    indices_count: model.read().unwrap().indices.len(),
    //model: model,
    material_buffer: vec![0.0; 10],
    materials: materials,
  }))
}

pub struct ImplDrawable {
  vao: GLuint,
  indices: VBO,
  vertices: VBO,
  normals: VBO,
  // textures: VBO,
  transforms: VBO,
  transform_buffer: Vec<f32>,
  pub references: HashMap<u64, Node>,
  shader: Shader,
  indices_count: usize,
  //model: Model,
  materials: VBO,
  material_buffer: Vec<f32>,
}

impl ImplDrawable {
  pub fn id(&self) -> GLuint {
    self.vao
  }

  pub fn update_instance_matrices(&mut self) {
    if self.transform_buffer.len() != self.references.len() * 16 {
      self.transform_buffer = vec![0.0; self.references.len() * 16];
      self.material_buffer = vec![0.0; self.references.len() * 10];
    }

    let mut off = 0;
    let mut mat_off = 0;
    for node in &self.references {
      let n = node.1.read().unwrap();
      {
        let m = n.world_transform.lock().unwrap();
        self.transform_buffer[off .. off + 16].copy_from_slice(&*m);
        off += 16;
      }
      {
        let mat = n.material.read().unwrap();
        self.material_buffer[mat_off .. mat_off + 3].copy_from_slice(&mat.ambient);
        self.material_buffer[mat_off + 3.. mat_off + 6].copy_from_slice(&mat.diffuse);
        self.material_buffer[mat_off + 6.. mat_off + 9].copy_from_slice(&mat.specular);
        self.material_buffer[mat_off + 9] = mat.shininess;
        mat_off += 10;
      }
    }

    self.transforms.set(&self.transform_buffer);
    self.materials.set(&self.material_buffer);
  }

  pub fn draw(&mut self, mvp: &Matrix) {
    self.update_instance_matrices();
    unsafe {
      gl::BindVertexArray(self.vao);
    }

    let shader = self.shader.read().unwrap();

    // index buffer has to be present, because it is not bound via context of vertex attribute diffenition
    self.indices.bind();

    shader.bind();

    unsafe {
      {
        let mvp = mvp.lock().unwrap();
        gl::UniformMatrix4fv(shader.mvp, 1, gl::FALSE, mvp.as_ptr() as *const _);
      }

      // !! meditate about it !!
      // https://stackoverflow.com/questions/32447641/what-is-common-cause-of-range-out-of-bounds-of-buffer-in-webgl
      gl::DrawElementsInstanced(
        gl::TRIANGLES,
        self.indices_count as GLint,
        gl::UNSIGNED_INT,
        std::ptr::null(),
        self.references.len() as GLint,
      );
    }
  }
}

pub type Drawable = Arc<RwLock<ImplDrawable>>;


pub struct DrawableUserData {
  pub drawable: Drawable,
}

impl mlua::UserData for DrawableUserData {}

pub fn load_module(lua: &mlua::Lua, ns: &mlua::Table) -> Result<(), mlua::Error> {
  use crate::methatron::error;
  let module = lua.create_table()?;

  let quick_load = lua.create_function(|_, (v, f, m): (String, String, String)| {
    let model = crate::methatron::model::load(&m).map_err(|e| error::to_lua_err(&e))?;
    let shader = crate::methatron::shader::new();

    {
      let vertex_src = std::fs::read_to_string(v)?;
      let fragment_src = std::fs::read_to_string(f)?;
      let mut shader = shader.write().map_err(|e| error::to_lua_err(&e.to_string()))?;
      shader.load(gl::VERTEX_SHADER, vertex_src).map_err(|e| error::to_lua_err(&e))?;
      shader.load(gl::FRAGMENT_SHADER, fragment_src).map_err(|e| error::to_lua_err(&e))?;
      shader.link().map_err(|e| error::to_lua_err(&e))?;
    }

    let drawable = crate::methatron::drawable::new(shader, model);
    let userdata = DrawableUserData {drawable: drawable};
    Ok(userdata)
  })?;
  module.set("quick_load", quick_load)?;
  ns.set("drawable", module)?;

  Ok(())
}
