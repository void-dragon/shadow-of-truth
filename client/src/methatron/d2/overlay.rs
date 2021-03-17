use std::sync::{Arc, RwLock};

use crate::methatron::math::matrix;

pub struct ImplOverlay {
  mvp: [f32; 16],
}

impl ImplOverlay {}

type Overlay = Arc<RwLock<ImplOverlay>>;

pub fn new(width: u32, height: u32) -> Overlay {
    let overlay = ImplOverlay {
        mvp: matrix::ortho(0.0, height as f32, 0.0, width as f32, 0.0, 1.0),
    };

    Arc::new(RwLock::new(overlay))
}

pub struct OverlayUserData(pub Overlay);

impl mlua::UserData for OverlayUserData {
  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_method("draw", |_, this, (name, filename): (String, String)| Ok(()));
  }
}

pub fn load_module(lua: &mlua::Lua, ns: &mlua::Table) -> Result<(), mlua::Error> {
  let module = lua.create_table()?;

  let lua_new = lua.create_function(|_, (width, height): (u32, u32)| {
    Ok(OverlayUserData(new(width, height)))
  })?;
  module.set("new", lua_new)?;

  ns.set("overlay", module)?;

  Ok(())
}