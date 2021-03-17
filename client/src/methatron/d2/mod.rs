pub mod overlay;
pub mod text;

pub fn load_module(lua: &mlua::Lua, ns: &mlua::Table) -> Result<(), mlua::Error> {
  let module = lua.create_table()?;

  overlay::load_module(lua, &module)?;
  text::load_module(lua, &module)?;

  ns.set("d2", module)?;

  Ok(())
}