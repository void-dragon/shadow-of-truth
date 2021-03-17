use std::error::Error;
use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc,
};

use crate::methatron;
use crate::context;
use crate::tracer;

fn lua_env(lua: &mlua::Lua) -> mlua::Result<Arc<AtomicBool>> {
  let l = lua.create_table()?;

  let print = lua.create_function(|_, params: mlua::Variadic<String>| {
    log::info!("{}", params.iter().fold("".to_owned(), |a, b| a + b));
    Ok(())
  })?;
  l.set("print", print)?;

  let running = Arc::new(AtomicBool::new(true));
  {
    let running = running.clone();
    let exe = lua.create_function(move |_, (): ()| {
      running.store(false, Ordering::SeqCst);
      Ok(())
    })?;
    l.set("exit", exe)?;
  }

  l.set("tracer", tracer::new())?;

  lua.globals().set("lua", l)?;

  Ok(running)
}

pub fn execute<F>(ctx: context::Context, filename: &str, env: F) -> Result<(), Box<dyn Error>> 
where F: Fn(&mlua::Table) -> mlua::Result<()>
{
  let lua = mlua::Lua::new();
  let meth = lua.create_table()?;

  methatron::scene::load_module(&lua, &meth)?;
  methatron::node::load_module(&lua, &meth)?;
  methatron::math::load_module(&lua, &meth)?;
  methatron::light::load_module(&lua, &meth)?;
  methatron::d2::load_module(&lua, &meth)?;

  let globals = lua.globals();

  env(&globals)?;
  let running = lua_env(&lua)?;

  globals.set("methatron", meth)?;
  globals.set("engine", context::ContextUserData(ctx.clone()))?;

  {
    let src = std::fs::read(filename)?;
    let code = lua.load(&src);
    code.exec()?;
  }

  if globals.contains_key("on_update")? {
    let on_update: mlua::Function = globals.get("on_update")?;
    let events = crate::events::get();

    while running.load(Ordering::SeqCst) {
      let start = std::time::Instant::now();

      while let Ok(event) = events.receiver.try_recv() {
        let cb: Option<mlua::Function> = match event {
          crate::events::Events::Connected => {
            globals.get("on_connect").ok()
          }
          crate::events::Events::Disconnected => {
            globals.get("on_connect").ok()
          }
          crate::events::Events::KeyPressed(key) => {
            globals.get("on_key_press").ok().map(|f: mlua::Function| f.bind(key).unwrap())
          }
          crate::events::Events::KeyReleased(key) => {
            globals.get("on_key_release").ok().map(|f: mlua::Function| f.bind(key).unwrap())
          }
          crate::events::Events::MouseWheel(pos) => {
            globals.get("on_mouse_wheel").ok().map(|f: mlua::Function| f.bind(pos).unwrap())
          }
          _ => {None}
        };

        if let Some(cb) = cb {
          cb.call(())?;
        }
      }

      on_update.call(())?;
      let elapsed = start.elapsed();

      if elapsed.as_millis() < 30 {
        std::thread::sleep(std::time::Duration::from_millis(30) - elapsed);
      }
    }
  }

  Ok(())
}