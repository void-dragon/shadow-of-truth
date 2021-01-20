pub mod matrix;
pub mod vector;

pub fn load_module(lua: &mlua::Lua, ns: &mlua::Table) -> Result<(), mlua::Error> {
  let module = lua.create_table()?;

  matrix::load_module(lua, &module)?;
  vector::load_module(lua, &module)?;

  ns.set("math", module)?;

  Ok(())
}
