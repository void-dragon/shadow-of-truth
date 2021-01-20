use std::sync::Arc;

pub struct MethatronError {
  message: String,
}

impl std::fmt::Display for MethatronError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.message)
  }
}

impl std::fmt::Debug for MethatronError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.message)
  }
}

impl std::error::Error for MethatronError {}

pub fn to_lua_err(msg: &str) -> mlua::Error {
    mlua::Error::ExternalError(Arc::new(MethatronError { message: msg.to_string() }))
}