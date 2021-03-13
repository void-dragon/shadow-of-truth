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
  }))
}


pub struct ImplShader {
  pub program: GLuint,
  sources: Vec<GLuint>,
  is_linked: bool,
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
            Ok(())
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
        Ok(_) => {
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

  pub fn get_attribute_location(&self, name: String) -> u32 {
    let pump = pump::get();
    let program = self.program;

    let position = pump.exec(move || {
      let cname = CString::new(name.as_bytes()).unwrap();
      let position: u32 = unsafe { gl::GetAttribLocation(program, cname.as_ptr()) as _ };
      position
    });

    position
  }

  pub fn get_uniform_location(&self, name: String) -> i32 {
    let pump = pump::get();
    let program = self.program;

    let position = pump.exec(move || {
      let cname = CString::new(name.as_bytes()).unwrap();
      let position = unsafe { gl::GetUniformLocation(program, cname.as_ptr()) };
      position
    });

    position
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
