pub type Vector = [f32; 3];

pub struct VectorUserData {
  pub vector: Vector
}

impl mlua::UserData for VectorUserData {}

pub fn add(a: &Vector, b: &Vector) -> Vector {
  return [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
}

pub fn sub(a: &Vector, b: &Vector) -> Vector {
  return [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
}

pub fn mul_by_scalar(v: &Vector, s: f32) -> Vector {
  return [v[0] * s, v[1] * s, v[2] * s];
}

pub fn div_by_scalar(v: &Vector, s: f32) -> Vector {
  return [v[0] / s, v[1] / s, v[2] / s];
}

pub fn angle(w: &Vector, v: &Vector) -> f32 {
  let d = w[0] * v[0] + w[1] * v[1] + w[2] * v[2];
  let a = w[1] * v[2] - w[2] * v[1];
  let b = w[2] * v[0] - w[0] * v[2];
  let c = w[0] * v[1] - w[1] * v[0];

  return (d / (a * a + b * b + c * c).sqrt()).acos();
}

pub fn dot(w: &Vector, v: &Vector) -> f32 {
  return w[0] * v[0] + w[1] * v[1] + w[2] * v[2];
}

pub fn cross(w: &Vector, v: &Vector) -> Vector {
  return [w[1] * v[2] - w[2] * v[1], w[2] * v[0] - w[0] * v[2], w[0] * v[1] - w[1] * v[0]];
}

pub fn distance(w: &Vector, v: &Vector) -> f32 {
  let dx = w[0] - v[0];
  let dy = w[1] - v[1];
  let dz = w[2] - v[2];

  return (dx * dx + dy * dy + dz * dz).sqrt();
}

pub fn magnitude(w: &Vector) -> f32 {
  return (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt();
}

pub fn normalize(w: &mut Vector) {
  let sum = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt();

  w[0] /= sum;
  w[1] /= sum;
  w[2] /= sum;
}

pub fn lerp(w: &mut Vector, end: &Vector, percent: f32) {
  w[0] = w[0] + percent * (end[0] - w[0]);
  w[1] = w[1] + percent * (end[1] - w[1]);
  w[2] = w[2] + percent * (end[2] - w[2]);
}

pub fn slerp(w: &mut Vector, end: &Vector, percent: f32) {
  let _dot = dot(w, end);
  let theta = _dot.acos() * percent;
  let relx = end[0] - w[0] * _dot;
  let rely = end[1] - w[1] * _dot;
  let relz = end[2] - w[2] * _dot;
  let ctheta = theta.cos();
  let stheta = theta.sin();

  w[0] = w[0] * ctheta + relx * stheta;
  w[1] = w[1] * ctheta + rely * stheta;
  w[2] = w[2] * ctheta + relz * stheta;
}

pub fn load_module(lua: &mlua::Lua, ns: &mlua::Table) -> Result<(), mlua::Error> {
  let module = lua.create_table()?;

  let lua_new = lua.create_function(|_, (x, y, z): (f32, f32, f32)| {
    Ok(VectorUserData { vector: [x, y, z]})
  })?;
  module.set("new", lua_new)?;

  ns.set("vector", module)?;

  Ok(())
}
