use std::error::Error;

use crate::methatron;
use crate::context;
use crate::tracer;

pub fn execute(ctx: context::Context, filename: &str) -> Result<(), Box<dyn Error>> {
  let lua = mlua::Lua::new();
  let meth = lua.create_table()?;

  methatron::scene::load_module(&lua, &meth)?;
  methatron::drawable::load_module(&lua, &meth)?;
  methatron::node::load_module(&lua, &meth)?;
  methatron::math::load_module(&lua, &meth)?;

  let globals = lua.globals();
  globals.set("methatron", meth)?;
  globals.set("context", context::ContextUserData(ctx.clone()))?;
  globals.set("tracer", tracer::new())?;

  let print = lua.create_function(|_, params: mlua::Variadic<String>| {
    log::info!("{}", params.iter().fold("".to_owned(), |a, b| a + b));
    Ok(())
  })?;
  globals.set("print", print)?;

  {
    let ctx = ctx.clone();
    let exe = lua.create_function(move |_, filename: String| {
      let ctx = ctx.clone();

      // tokio::spawn(async move {
      //   if let Err(e) = execute(ctx, &filename).await {
      //     log::error!("{}", e.to_string());
      //   }
      // });

      Ok(())
    })?;
    globals.set("execute", exe)?;
  }

  {
    let src = std::fs::read(filename)?;
    let code = lua.load(&src);
    code.exec()?;
  }

  if globals.contains_key("on_update")? {
    let on_update: mlua::Function = globals.get("on_update")?;
    let events = crate::events::get();

    loop {
      let start = std::time::Instant::now();
      while let Ok(event) = events.receiver.try_recv() {
        match event {
          crate::events::Events::Connected => {
            let on_connect: mlua::Function = globals.get("on_connect")?;
            on_connect.call(())?;
          }
          _ => {}
        }
      }
      on_update.call(())?;
      let elapsed = start.elapsed();

      std::thread::sleep(std::time::Duration::from_millis(30) - elapsed);
    }
  }
  else {
    Ok(())
  }
}