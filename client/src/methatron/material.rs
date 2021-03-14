use std::sync::{Arc, RwLock};

pub struct ImplMaterial {
  pub ambient: [f32; 3],
  pub diffuse: [f32; 3],
  pub specular: [f32; 3],
  pub shininess: f32,
}

pub type Material = Arc<RwLock<ImplMaterial>>;

pub fn new() -> Material {
  let m = ImplMaterial {
    ambient: [0.2, 0.6, 0.8],
    diffuse: [0.2, 0.6, 0.8],
    specular: [0.2, 0.6, 0.8],
    shininess: 10.0,
  };
  Arc::new(RwLock::new(m))
}

pub struct MaterialUserData(pub Material);

impl mlua::UserData for MaterialUserData {
  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_method("get_ambient", |_, this, ()| {
      let m = this.0.read().unwrap();
      Ok(m.ambient)
    });

    methods.add_method("get_diffuse", |_, this, ()| {
      let m = this.0.read().unwrap();
      Ok(m.diffuse)
    });

    methods.add_method("get_specular", |_, this, ()| {
      let m = this.0.read().unwrap();
      Ok(m.specular)
    });

    methods.add_method("get_shininess", |_, this, ()| {
      let m = this.0.read().unwrap();
      Ok(m.shininess)
    });

    methods.add_method("set_ambient", |_, this, v: Vec<f32>| {
      let mut m = this.0.write().unwrap();
      m.ambient[0] = v[0];
      m.ambient[1] = v[1];
      m.ambient[2] = v[2];
      Ok(())
    });

    methods.add_method("set_diffuse", |_, this, v: Vec<f32>| {
      let mut m = this.0.write().unwrap();
      m.diffuse[0] = v[0];
      m.diffuse[1] = v[1];
      m.diffuse[2] = v[2];
      Ok(())
    });

    methods.add_method("set_specular", |_, this, v: Vec<f32>| {
      let mut m = this.0.write().unwrap();
      m.specular[0] = v[0];
      m.specular[1] = v[1];
      m.specular[2] = v[2];
      Ok(())
    });

    methods.add_method("set_shininess", |_, this, v: f32| {
      let mut m = this.0.write().unwrap();
      m.shininess = v;
      Ok(())
    });
  }
}
