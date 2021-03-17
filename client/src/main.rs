use std::time::{Duration, Instant};

mod context;
mod events;
mod lua;
mod methatron;
mod network;
mod tracer;

fn check_gl_error(info: &str) {
  let error = unsafe { gl::GetError() };

  if error != 0 {
    panic!("OPEN_GL ERROR {} {}", error, info);
  }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let env = env_logger::Env::default().default_filter_or("debug");
  env_logger::Builder::from_env(env).init();

  let event_loop = glutin::event_loop::EventLoop::new();
  let window = glutin::window::WindowBuilder::new()
    .with_title("shadow-of-truth")
    .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
  let gl_window = glutin::ContextBuilder::new()
    .with_gl(glutin::GlRequest::Latest)
    .build_windowed(window, &event_loop)?;
  gl_window.window().set_cursor_visible(false);
  gl_window.window().set_cursor_grab(true)?;

  // It is essential to make the context current before calling `gl::load_with`.
  let gl_window = unsafe { gl_window.make_current() }.unwrap();

  // Load the OpenGL function pointers
  gl::load_with(|symbol| gl_window.get_proc_address(symbol));

  let ctx = context::get();
  let pump = methatron::pump::get();
  let ev = events::get();

  {
    let ctx = ctx.clone();
    std::thread::spawn(move || {
      log::info!("init luajit");

      if let Err(e) = lua::execute(ctx, "assets/scripts/init.lua", |_| Ok(())) {
        log::error!("{}", e.to_string());
      }
    });
  }

  log::info!("configure open-gl");
  unsafe {
    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
    // gl::Enable(gl::CULL_FACE);
    gl::Enable(gl::DEPTH_TEST);
    gl::DepthFunc(gl::LEQUAL);
    gl::Enable(gl::BLEND);
    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    gl::ClearColor(0.0, 0.0, 0.0, 1.0);
  }

  let mut start = Instant::now();
  event_loop.run(move |event, _, control_flow| {
    use glutin::event::{Event, WindowEvent, DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode, MouseScrollDelta};
    use glutin::event_loop::ControlFlow;
    let next = Instant::now() + Duration::from_millis(25);
    *control_flow = ControlFlow::WaitUntil(next);

    match event {
      Event::LoopDestroyed => return,
      Event::WindowEvent { event, .. } => match event {
          WindowEvent::CloseRequested => { *control_flow = ControlFlow::Exit }
          WindowEvent::KeyboardInput { 
            input: KeyboardInput { virtual_keycode: Some(virtual_code), state, .. },
            ..
          } => {
            if virtual_code == VirtualKeyCode::Escape {
              *control_flow = ControlFlow::Exit;
            }

            let name = format!("{:?}", virtual_code);
            let is_pressed = { ctx.read().unwrap().keys_down.get(&name).is_some() };
            match state {
              ElementState::Pressed => {
                if !is_pressed {
                  ev.sender.send(events::Events::KeyPressed(name.clone())).unwrap();
                }
                ctx.write().unwrap().keys_down.insert(name);
              }
              ElementState::Released => {
                if is_pressed {
                  ev.sender.send(events::Events::KeyReleased(name.clone())).unwrap();
                }
                ctx.write().unwrap().keys_down.remove(&name);
              }
            }
          }
          WindowEvent::MouseWheel{delta: MouseScrollDelta::LineDelta(x, y), ..} => {
            ev.sender.send(events::Events::MouseWheel(y)).unwrap();
          }
          WindowEvent::MouseWheel{delta, ..} => {
            log::debug!("{:?}", delta);
          }
          _ => {}
      },
      Event::DeviceEvent { event, .. } => {
        match event {
          DeviceEvent::MouseMotion{delta} => {
            let mut c = ctx.write().unwrap();
            c.mouse_position[0] += delta.0;
            c.mouse_position[1] += delta.1;
          }
          _ => {}
        }
      }
      _ => (),
    }

    if *control_flow == ControlFlow::Exit {
      ctx.read().unwrap().network.shutdown();
    }

    if start.elapsed().as_millis() > 20 {
      pump.run();

      check_gl_error("pump");

      if let Some(scene) = &ctx.read().unwrap().scene {
        scene.read().unwrap().draw();
      }

      check_gl_error("scene");


      check_gl_error("2d");

      gl_window.swap_buffers().unwrap();
      start = Instant::now();
    }
  });
  //Ok(())
}
