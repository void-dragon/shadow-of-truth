use std::hash::{Hash, Hasher};

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename = "COLLADA")]
pub struct Collada {
    pub library_geometries: LibraryGeometry,
}

#[derive(Deserialize)]
pub struct LibraryGeometry {
    #[serde(rename = "geometry", default)]
    pub geometires: Vec<Geometry>
}

#[derive(Deserialize)]
pub struct Geometry {
    pub id: String,
    pub name: String,
    pub mesh: Mesh,
}

#[derive(Deserialize)]
pub struct Mesh {
    #[serde(rename = "source")]
    pub sources: Vec<Source>,
    pub triangles: Triangles,
}

#[derive(Deserialize)]
pub struct Source {
    pub id: String,
    pub float_array: FloatArray,
}

#[derive(Deserialize)]
pub struct FloatArray {
    pub id: String,
    pub count: u32,
    #[serde(rename = "$value")]
    pub body: String,
}

impl FloatArray {
    pub fn as_vec(&self) -> Vec<f32> {
        self.body.split(' ').map(|f| f.parse().unwrap()).collect()
    }
}

#[derive(Deserialize)]
pub struct Triangles {
    pub count: u32,
    #[serde(rename = "input")]
    pub inputs: Vec<TriangleInput>,
    #[serde(rename = "p")]
    pub indices: TriangelIndices,
}

impl Triangles {
    pub fn clean_indices(&self) -> Vec<CleanIndex> {
        let mut indices = Vec::new();

        let p: Vec<usize> = self.indices.body.split(' ').map(|v| v.parse().unwrap()).collect();
        let stride = self.inputs.len();

        let mut i = 0;
        while i < p.len() {
            indices.push(CleanIndex {
                vertex: p[i],
                normal: p[i + 1],
            });
            i += stride;
        }

        indices
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct CleanIndex {
    pub vertex: usize,
    pub normal: usize,
}

impl Hash for CleanIndex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vertex.hash(state);
        self.normal.hash(state);
    }
}

#[derive(Deserialize)]
pub struct TriangleInput {
    pub semantic: String,
    pub source: String,
    pub offset: u32,
}

#[derive(Deserialize)]
pub struct TriangelIndices {
    #[serde(rename = "$value")]
    pub body: String,
}