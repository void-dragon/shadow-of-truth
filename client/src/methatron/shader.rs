use gl::types::{GLchar, GLenum, GLint, GLuint};
use log::debug;

use std::ffi::CString;
use std::sync::{Arc, RwLock};

use crate::methatron::pump;

pub fn new() -> Shader {
  let pump = pump::get();
  let program = pump.exec(|| unsafe { gl::CreateProgram() });
  debug!("created shader {}", program);

  Arc::new(RwLock::new(ImplShader {
    program: program,
    sources: Vec::new(),
    is_linked: false,
    mvp: 0,
    material: MaterialLoc {
      ambient: 0,
      diffuse: 0,
      specular: 0,
      shininess: 0,
    },
    position: 0,
    normal: 0,
    t0: 0,
  }))
}

#[derive(Debug)]
pub struct MaterialLoc {
  pub ambient: GLuint,
  pub diffuse: GLuint,
  pub specular: GLuint,
  pub shininess: GLuint,
}

pub struct ImplShader {
  pub program: GLuint,
  sources: Vec<GLuint>,
  is_linked: bool,
  pub mvp: GLint,
  pub material: MaterialLoc,
  pub position: GLuint,
  pub normal: GLuint,
  pub t0: GLuint,
}

impl ImplShader {
  pub fn load(&mut self, shader_type: GLenum, src: String) -> Result<(), String> {
    let pump = pump::get();

    let result = pump.exec(move || {
      let mut success = gl::FALSE as GLint;
      let shader = unsafe {
        let shader = gl::CreateShader(shader_type);
        let c_src = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_src.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);

        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

        shader
      };

      if success == gl::TRUE as GLint {
        Ok(shader)
      } else {
        let mut len = 0;

        unsafe {
          gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
          let mut buf = Vec::with_capacity(len as usize);
          buf.set_len((len as usize) - 1);
          gl::GetShaderInfoLog(shader, len, std::ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);

          Err(String::from_utf8(buf).expect("could not convert shader error message"))
        }
      }
    });

    match result {
      Ok(shader) => {
        debug!("successfull loaded source {}", shader);
        self.sources.push(shader);
        Ok(())
      }
      Err(e) => Err(e)
    }
  }

  pub fn link(&mut self) -> Result<(), String> {
    if self.is_linked {
      Err("shader already linked".to_string())
    } else {
      let data = (self.program, self.sources.clone());
      let pump = pump::get();

      debug!("link shader {}", self.program);
      let result = pump.exec(move || {
        let (program, sources) = &data;
        let program = *program;

        unsafe {
          for source in sources {
            gl::AttachShader(program, *source);
          }
          gl::LinkProgram(program);

          let mut status = gl::FALSE as GLint;
          gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

          if status == gl::TRUE as GLint {
            let mvp = CString::new("mvp".as_bytes()).unwrap();
            let mvp = gl::GetUniformLocation(program, mvp.as_ptr());

            let mat_ambient = CString::new("m_ambient".as_bytes()).unwrap();
            let amb = gl::GetAttribLocation(program, mat_ambient.as_ptr()) as _;
            let material = MaterialLoc {
              ambient: amb,
              diffuse: amb + 1,
              specular: amb + 2,
              shininess: amb + 3,
            };

            let pos = CString::new("position".as_bytes()).unwrap();
            let position = gl::GetAttribLocation(program, pos.as_ptr()) as _;

            let normal = CString::new("normal".as_bytes()).unwrap();
            let normal = gl::GetAttribLocation(program, normal.as_ptr()) as _;

            let t0 = CString::new("t0".as_bytes()).unwrap();
            let t0 = gl::GetAttribLocation(program, t0.as_ptr()) as _;

            Ok((mvp, material, position, normal, t0))
          } else {
            let mut len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1);
            gl::GetProgramInfoLog(program, len, std::ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);

            Err(String::from_utf8(buf).expect("could not convert shader program error message"))
          }
        }
      });

      match result {
        Ok((mvp, material, pos, normal, t0)) => {
          log::debug!("{:?}", material);
          self.mvp = mvp;
          self.material = material;
          self.position = pos;
          self.normal = normal;
          self.t0 = t0;
          self.is_linked = true;

          Ok(())
        }
        Err(e) => Err(e)
      }
    }
  }

  pub fn bind(&self) {
    unsafe {
      gl::UseProgram(self.program);
    }
  }
}

impl Drop for ImplShader {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteProgram(self.program);
    }
  }
}

pub type Shader = Arc<RwLock<ImplShader>>;
