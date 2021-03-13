use crate::methatron::pump;

pub struct Shadow {
    pub fbo: u32,
    pub depth_map: u32,
    pub width: u32,
    pub height: u32,
}

pub fn new(width: i32, height: i32) -> Shadow {
    let pump = pump::get();
    
    let (fbo, depth_map) = pump.exec(move || {
        let mut fbo = 0;
        let mut depth_map = 0;

        unsafe {
            gl::GenFramebuffers(1, &mut fbo as *mut _);
            gl::GenTextures(1, &mut depth_map as *mut _);
            gl::BindTexture(gl::TEXTURE_2D, depth_map);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::DEPTH_COMPONENT as i32, width, height, 0, gl::DEPTH_COMPONENT, gl::FLOAT, std::ptr::null());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as i32); 
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as i32);
            gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, [1.0, 1.0, 1.0, 1.0].as_ptr());

            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, depth_map, 0);
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        (fbo, depth_map)
    });

    Shadow {
        fbo: fbo,
        depth_map: depth_map,
        width: width as _,
        height: height as _,
    }
}