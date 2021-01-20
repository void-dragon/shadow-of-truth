use serde::{Deserialize, Serialize};

use std::fs::File;
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize)]
pub struct ImplModel {
  pub name: String,
  pub indices: Vec<u32>,
  pub vertices: Vec<f32>,
  pub normals: Vec<f32>,
}

pub type Model = Arc<RwLock<ImplModel>>;

pub fn load(name: &str) -> Result<Model, String> {
  let reader = File::open(name).map_err(|e| e.to_string())?;
  let model: ImplModel = serde_json::from_reader(reader).map_err(|e| e.to_string())?;
  Ok(Arc::new(RwLock::new(model)))
}
