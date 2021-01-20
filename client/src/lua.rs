use log::info;

use crate::methatron;
use crate::context;
use crate::tracer;

pub fn execute(ctx: context::Context, filename: &str) -> mlua::prelude::LuaResult<()> {
  let lua = mlua::Lua::new();
  let meth = lua.create_table()?;

  methatron::scene::load_module(&lua, &meth)?;
  methatron::drawable::load_module(&lua, &meth)?;
  methatron::node::load_module(&lua, &meth)?;
  methatron::math::load_module(&lua, &meth)?;

  let globals = lua.globals();
  globals.set("methatron", meth)?;
  globals.set("context", context::ContextUserData { context: ctx })?;
  globals.set("tracer", tracer::new())?;

  let print = lua.create_function(|_, params: mlua::Variadic<String>| {
    info!("{}", params.iter().fold(" ".to_owned(), |a, b| a + b));
    Ok(())
  })?;
  globals.set("print", print)?;

  lua.load(&std::fs::read(filename)?).exec()?;
  let on_update: mlua::Function = globals.get("on_update")?;

  loop {
    on_update.call::<_, ()>(()).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(25));
  }
}