use std::sync::{Arc, RwLock};

pub struct ImplMaterial {
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub shininess: f32,
}

pub type Material = Arc<RwLock<ImplMaterial>>;

pub fn new() -> Material {
    let m = ImplMaterial {
        ambient: [0.2, 0.6, 0.8],
        diffuse: [0.2, 0.6, 0.8],
        specular: [0.2, 0.6, 0.8],
        shininess: 10.0,
    };
    Arc::new(RwLock::new(m))
}