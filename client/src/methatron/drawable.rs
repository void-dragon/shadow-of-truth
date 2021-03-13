use gl::types::GLuint;

use crate::methatron::{
  pump, 
  model::Model, 
  node::Node, 
  vbo::VBO,
  scene::DrawAttribLocations,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub fn new(locs: DrawAttribLocations, model: Model) -> Drawable {
  let pump = pump::get();

  let (vao, indices, vertices, normals, textures, transforms, materials) = {
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
        gl::EnableVertexAttribArray(locs.position);
        gl::VertexAttribPointer(locs.position as _, 3, gl::FLOAT, gl::FALSE, float_size * 3, std::ptr::null());
      }

      let normals = VBO::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW, &model.normals);

      unsafe {
        gl::EnableVertexAttribArray(locs.normal);
        gl::VertexAttribPointer(locs.normal, 3, gl::FLOAT, gl::FALSE, float_size * 3, std::ptr::null());
      }

      let textures = VBO::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW, &model.texcoords);
      unsafe {
        gl::EnableVertexAttribArray(locs.texcoords);
        gl::VertexAttribPointer(locs.texcoords, 2, gl::FLOAT, gl::FALSE, float_size * 2, std::ptr::null());
      }

      let transform_buffer: Vec<f32> = vec![0.0; 16];
      let transforms = VBO::new(gl::ARRAY_BUFFER, gl::DYNAMIC_DRAW, &transform_buffer);

      let matrix_offset = float_size * 16;

      unsafe {
        gl::EnableVertexAttribArray(locs.t0 + 0);
        gl::EnableVertexAttribArray(locs.t0 + 1);
        gl::EnableVertexAttribArray(locs.t0 + 2);
        gl::EnableVertexAttribArray(locs.t0 + 3);
        gl::VertexAttribPointer(locs.t0 + 0, 4, gl::FLOAT, gl::FALSE, matrix_offset, 0 as *const _);
        gl::VertexAttribPointer(locs.t0 + 1, 4, gl::FLOAT, gl::FALSE, matrix_offset, (float_size * 4) as *const _);
        gl::VertexAttribPointer(locs.t0 + 2, 4, gl::FLOAT, gl::FALSE, matrix_offset, (float_size * 8) as *const _);
        gl::VertexAttribPointer(locs.t0 + 3, 4, gl::FLOAT, gl::FALSE, matrix_offset, (float_size * 12) as *const _);
        gl::VertexAttribDivisor(locs.t0 + 0, 1);
        gl::VertexAttribDivisor(locs.t0 + 1, 1);
        gl::VertexAttribDivisor(locs.t0 + 2, 1);
        gl::VertexAttribDivisor(locs.t0 + 3, 1);
      }

      let material_buffer: Vec<f32> = vec![1.0; 10];
      let materials = VBO::new(gl::ARRAY_BUFFER, gl::DYNAMIC_DRAW, &material_buffer);
      let material_offset = float_size * 10;
      unsafe {
        gl::EnableVertexAttribArray(locs.material.ambient);
        gl::EnableVertexAttribArray(locs.material.diffuse);
        gl::EnableVertexAttribArray(locs.material.specular);
        gl::EnableVertexAttribArray(locs.material.shininess);
        gl::VertexAttribPointer(locs.material.ambient, 3, gl::FLOAT, gl::FALSE, material_offset, 0 as *const _);
        gl::VertexAttribPointer(locs.material.diffuse, 3, gl::FLOAT, gl::FALSE, material_offset, (float_size * 3) as *const _);
        gl::VertexAttribPointer(locs.material.specular, 3, gl::FLOAT, gl::FALSE, material_offset, (float_size * 6) as *const _);
        gl::VertexAttribPointer(locs.material.shininess, 1, gl::FLOAT, gl::FALSE, material_offset, (float_size * 9) as *const _);
        gl::VertexAttribDivisor(locs.material.ambient, 1);
        gl::VertexAttribDivisor(locs.material.diffuse, 1);
        gl::VertexAttribDivisor(locs.material.specular, 1);
        gl::VertexAttribDivisor(locs.material.shininess, 1);
      }

      (vao, indices, vertices, normals, textures, transforms, materials)
    })
  };
  log::debug!("created drawable {}", vao);

  Arc::new(RwLock::new(ImplDrawable {
    vao: vao,
    indices: indices,
    vertices: vertices,
    normals: normals,
    textures: textures,
    transforms: transforms,
    transform_buffer: vec![0.0; 16],
    references: HashMap::new(),
    indices_count: model.read().unwrap().indices.len(),
    //model: model,
    material_buffer: vec![0.0; 10],
    materials: materials,
  }))
}

pub struct ImplDrawable {
  pub vao: GLuint,
  pub indices: VBO,
  vertices: VBO,
  normals: VBO,
  textures: VBO,
  transforms: VBO,
  transform_buffer: Vec<f32>,
  pub references: HashMap<u64, Node>,
  pub indices_count: usize,
  //model: Model,
  materials: VBO,
  material_buffer: Vec<f32>,
}

impl ImplDrawable {
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
}

impl Drop for ImplDrawable {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteVertexArrays(1, &mut self.vao as *mut _);
    }
  }
}

pub type Drawable = Arc<RwLock<ImplDrawable>>;

pub struct DrawableUserData(pub Drawable);

impl mlua::UserData for DrawableUserData {}
