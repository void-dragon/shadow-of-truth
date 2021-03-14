use crate::network::{self, Network};

use std::collections::BTreeSet;
use std::sync::{Arc, RwLock};

use crate::methatron::scene::{Scene, SceneUserData};

pub struct ImplContext {
  pub scene: Option<Scene>,
  pub keys_down: BTreeSet<String>,
  pub mouse_position: [f64; 2],
  pub network: Network,
}

pub type Context = Arc<RwLock<ImplContext>>;

static mut CONTEXT: Option<Context> = None;

pub fn get() -> Context {
  unsafe {
    if let Some(ref p) = CONTEXT {
      p.clone()
    }
    else {
      let p = Arc::new(RwLock::new(ImplContext {
        scene: None,
        keys_down: BTreeSet::new(),
        mouse_position: [0.0, 0.0],
        network: network::new(),
      }));
      CONTEXT = Some(p.clone());
      p
    }
  }
}

pub struct ContextUserData(pub Context);

impl mlua::UserData for ContextUserData {
   fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
      methods.add_method("set_scene", |_, this, scene: mlua::AnyUserData| {
        let mut ctx = this.0.write().unwrap();
        let bscene = scene.borrow::<SceneUserData>().unwrap();
        let scene = bscene.scene.clone();

        ctx.scene = Some(scene);
        Ok(())
      });

      methods.add_method("get_scene", |l, this, ()| {
        use mlua::ToLua;

        let ctx = this.0.read().unwrap();
        if let Some(ref scene) = ctx.scene {
          Ok(SceneUserData{scene: scene.clone()}.to_lua(&l)?)
        }
        else {
          Ok(mlua::Value::Nil)
        }
      });

      methods.add_method("is_key_down", |_, this, key: String| {
        let ctx = this.0.read().unwrap();
        Ok(ctx.keys_down.contains(&key))
      });

      methods.add_method("mouse_position", |_, this, (): ()| {
        let ctx = this.0.read().unwrap();
        Ok(ctx.mouse_position)
      });

      methods.add_method("network", |_, this, (): ()| {
        let ctx = this.0.read().unwrap();
        Ok(ctx.network.clone())
      });

      methods.add_method("execute", |_, this, filename: String| {
        Ok(())
      });
   }
}
