use gl::types::{GLenum, GLuint};

#[derive(Clone)]
pub struct VBO {
  id: GLuint,
  target: GLenum,
  draw_type: GLenum,
}

impl VBO {
  pub fn new<T>(target: GLenum, draw_type: GLenum, data: &Vec<T>) -> VBO {
    let mut id = 0;
    unsafe {
      gl::CreateBuffers(1, &mut id as *mut _);
      gl::BindBuffer(target, id);
      gl::BufferData(
        target,
        (std::mem::size_of::<T>() * data.len()) as isize,
        std::mem::transmute(data.as_ptr()),
        draw_type,
      );
    }

    VBO {
      id: id,
      target: target,
      draw_type: draw_type,
    }
  }

  pub fn bind(&self) {
    unsafe {
      gl::BindBuffer(self.target, self.id);
    }
  }

  pub fn set<T>(&self, data: &Vec<T>) {
    unsafe {
      gl::BindBuffer(self.target, self.id);
      gl::BufferData(
        self.target,
        (std::mem::size_of::<T>() * data.len()) as isize,
        std::mem::transmute(data.as_ptr()),
        self.draw_type,
      );
    }
  }
}

impl Drop for VBO {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteBuffers(1, &self.id as *const _);
    }
  }
}
