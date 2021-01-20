use log::error;

use std::collections::BTreeSet;
use std::sync::{Arc, RwLock};

use crate::methatron::scene::{Scene, SceneUserData};

pub struct ImplContext {
  pub scene: Option<Scene>,
  pub keys_down: BTreeSet<String>,
}

pub type Context = Arc<RwLock<ImplContext>>;

pub fn new() -> Context {
  let ctx = Arc::new(RwLock::new(ImplContext {
    scene: None,
    keys_down: BTreeSet::new(),
  }));

  ctx
}

pub struct ContextUserData {
  pub context: Context,
}

impl mlua::UserData for ContextUserData {
   fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
      methods.add_method("set_scene", |_, this, scene: mlua::AnyUserData| {
        let mut ctx = this.context.write().unwrap();
        let bscene = scene.borrow::<SceneUserData>().unwrap();
        let scene = bscene.scene.clone();

        let root = scene.read().unwrap().root.clone();
        if let Err(e) = root.read().unwrap().on_load() {
          error!("{}", e.to_string());
        }

        ctx.scene = Some(scene);
        Ok(())
      });

      methods.add_method("is_key_down", |_, this, key: String| {
        let ctx = this.context.read().unwrap();
        Ok(ctx.keys_down.contains(&key))
      });
   }
}
