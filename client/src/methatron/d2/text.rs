use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::methatron::{
  pump,
  shader::{self, Shader},
};

struct Character {
  id: char,
  tid: u32,
  buffer: Vec<u8>,
  bounding: [i32; 2],
  bearing: [i32; 2],
}

pub struct Text {
  txt: String,
  x: f32,
  y: f32,
  scale: f32,
  color: [f32; 3],
}

pub struct ImplFont {
  shader: Shader,
  characters: HashMap<char, Character>,
  texts: Vec<Text>,
  vao: u32,
  vbo: u32,
  text_color: i32,
  projection: i32,
}

impl ImplFont {
  // pub fn draw(&self, mvp: &[f32; 16]) {
  //   self.shader.read().unwrap().bind();
  //   for text in &self.texts {
  //     let mut xoff = 0.0;
  //     unsafe {
  //       gl::Uniform3f(self.text_color, 1.0, 0.5, 0.1);
  //       gl::UniformMatrix4fv(self.projection, 1, gl::FALSE, mvp.as_ptr() as *const _);

  //       gl::ActiveTexture(gl::TEXTURE0);
  //       gl::BindVertexArray(self.vao);

  //       for c in text.txt.chars() {
  //         let ch = self.characters.get(&c).unwrap();

  //         let xpos = text.x + xoff;
  //         let ypos = text.y - ch.bearing[1] as f32 * text.scale;

  //         let w = ch.bounding[0] as f32 * text.scale;
  //         let h = ch.bounding[1] as f32 * text.scale;
  //         let vertices = [
  //           xpos,     ypos + h, 0.0, 1.0,
  //           xpos,     ypos,     0.0, 0.0,
  //           xpos + w, ypos,     1.0, 0.0,
  //           xpos,     ypos + h, 0.0, 1.0,
  //           xpos + w, ypos,     1.0, 0.0,
  //           xpos + w, ypos + h, 1.0, 1.0,
  //         ];
  //         gl::BindTexture(gl::TEXTURE_2D, ch.tid);
  //         gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
  //         gl::BufferSubData(gl::ARRAY_BUFFER, 0, 4 * 6 * 4, vertices.as_ptr() as _);
  //         gl::DrawArrays(gl::TRIANGLES, 0, 6);
  //         xoff += ch.bounding[0] as f32 * text.scale;
  //       }
  //     }
  //   }
  // }
}

impl Drop for ImplFont {
  fn drop(&mut self) {
    log::debug!("delete font");
  }
}

type Font = Arc<RwLock<ImplFont>>;

pub fn new(filename: String) -> Font {
  let pump = pump::get();
  let data = std::fs::read(filename).unwrap();
  let font = rusttype::Font::try_from_vec(data).unwrap();
  let scale = rusttype::Scale::uniform(32.0);
  let v_metrics = font.v_metrics(scale);
  let txt = Arc::new(RwLock::new(ImplFont {
    shader: shader::new(),
    characters: HashMap::new(),
    texts: Vec::new(),
    vao: 0,
    vbo: 0,
    text_color: 0,
    projection: 0,
  }));

  {
    let mut text = txt.write().unwrap();
    let vertex_src = std::fs::read_to_string("assets/shaders/text.vertex.glsl").unwrap();
    let fragment_src = std::fs::read_to_string("assets/shaders/text.fragment.glsl").unwrap();
    let (color, proj) = {
      let mut shader = text.shader.write().unwrap();
      shader.load(gl::VERTEX_SHADER, vertex_src).unwrap();
      shader.load(gl::FRAGMENT_SHADER, fragment_src).unwrap();
      shader.link().unwrap();
      (
        shader.get_uniform_location("textColor".to_owned()),
        shader.get_uniform_location("projection".to_owned())
      )
    };
    text.text_color = color;
    text.projection = proj;
  }

  let text = txt.clone();

  
  pump.exec(move || {
    let mut text = text.write().unwrap();
    let tokens = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"?.,;:'[]{}()<>=/$%&*+#-_|";
    for i in tokens.chars() {
      let glyph = font.glyph(i).scaled(scale).positioned(rusttype::point(0.0, v_metrics.ascent));
      let bb = glyph.pixel_bounding_box().unwrap();
      let width = bb.width();
      let height = bb.height();
      let mut buffer: Vec<u8> = vec![0; (width * height) as usize];
      let mut texture = 0;

      glyph.draw(|x, y, o| {
        let idx = (width as u32 * y + x) as usize;
        buffer[idx] = (255.0 * o) as u8;
      });

      unsafe {
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexImage2D( gl::TEXTURE_2D, 0, gl::RED as _, width, height, 0, gl::RED, gl::UNSIGNED_BYTE, buffer.as_ptr() as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
      }

      text.characters.insert(
        i,
        Character {
          id: i,
          tid: texture,
          buffer: buffer,
          bounding: [width, height],
          bearing: [bb.min.x, bb.min.y],
        },
      );
    }
    unsafe {
      gl::GenVertexArrays(1, &mut text.vao);
      gl::BindVertexArray(text.vao);

      gl::GenBuffers(1, &mut text.vbo);
      gl::BindBuffer(gl::ARRAY_BUFFER, text.vbo);
      gl::BufferData(gl::ARRAY_BUFFER, 4 * 6 * 4, std::ptr::null(), gl::DYNAMIC_DRAW);
      gl::EnableVertexAttribArray(0);
      gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, 4 * 4, std::ptr::null());

      gl::BindVertexArray(0);
      gl::BindBuffer(gl::ARRAY_BUFFER, 0);
      gl::BindTexture(gl::TEXTURE_2D, 0);
    }
  });

  txt
}

pub struct FontUserData(pub Font);

impl mlua::UserData for FontUserData {
  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_method("draw", |_, this, (x, y, txt): (f32, f32, String)| {
      let mut font = this.0.write().unwrap();
      font.texts.push(Text {
        txt: txt,
        x: x,
        y: y,
        scale: 2.0,
        color: [1.0, 0.5, 0.5],
      });
      Ok(())
    });
  }
}

pub fn load_module(lua: &mlua::Lua, ns: &mlua::Table) -> Result<(), mlua::Error> {
  let module = lua.create_table()?;

  let lua_new = lua.create_function(|_, filename: String| {
    log::debug!("{}", filename);
    Ok(FontUserData(new(filename)))
  })?;
  module.set("new", lua_new)?;

  ns.set("font", module)?;

  Ok(())
}