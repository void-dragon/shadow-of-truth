use std::{
    collections::HashMap,
    error::Error,
    fs::File,
};

use clap::{Arg, App};

mod collada;
mod truth;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("dae to model")
        .arg(Arg::with_name("filename").short("f").long("file").required(true).takes_value(true))
        .arg(Arg::with_name("name").short("n").long("name").takes_value(true))
        .arg(Arg::with_name("json").short("j").long("json"))
        .arg(Arg::with_name("cbor").short("c").long("cbor"))
        .get_matches();

    let file = File::open(matches.value_of("filename").unwrap())?;
    let data: collada::Collada = serde_xml_rs::from_reader(file)?;

    for geometry in &data.library_geometries.geometires {
        let mut index_map = HashMap::new();
        let vertices = geometry.mesh.sources[0].float_array.as_vec();
        let normals = geometry.mesh.sources[1].float_array.as_vec();
        let texcoords = geometry.mesh.sources[2].float_array.as_vec();
        let mut model = truth::Model {
            name: geometry.name.clone(),
            indices: Vec::new(),
            vertices: Vec::new(),
            normals: Vec::new(),
            texcoords: Vec::new(),
        };
        
        for idx in geometry.mesh.triangles.clean_indices() {
            if let Some(n) = index_map.get(&idx) {
                model.indices.push(*n);
            }
            else {
                let i = model.vertices.len() / 3;
                index_map.insert(idx.clone(), i);
                model.indices.push(i);
                for t in 0..3 {
                    model.vertices.push(vertices[idx.vertex * 3 + t]);
                    model.normals.push(normals[idx.normal * 3 + t]);
                }
                for t in 0..2 {
                    model.texcoords.push(texcoords[idx.texcoord * 2 + t]);
                }
            }
        }

        let name = if matches.is_present("name") {
            matches.value_of("name").unwrap().to_owned()
        }
        else {
            geometry.name.clone()
        };

        if matches.is_present("json") {
            let out = File::create(&format!("{}.json", name))?;
            serde_json::to_writer_pretty(out, &model)?;
        }

        if matches.is_present("cbor") {
            let out = File::create(&format!("{}.cbor", name))?;
            serde_cbor::to_writer(out, &model)?;
        }
    }

    Ok(())
}
