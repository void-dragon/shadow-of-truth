use serde::Serialize;

#[derive(Serialize)]
pub struct Model {
    pub name: String,
    pub indices: Vec<usize>,
    pub vertices: Vec<f32>,
    pub normals: Vec<f32>,
}