use std::time::{Duration, Instant};

mod context;
mod events;
mod lua;
mod methatron;
mod network;
mod tracer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let env = env_logger::Env::default().default_filter_or("debug");
  env_logger::Builder::from_env(env).init();

  let event_loop = glutin::event_loop::EventLoop::new();
  let window = glutin::window::WindowBuilder::new()
    .with_title("shadow-of-truth");
  let gl_window = glutin::ContextBuilder::new()
    .build_windowed(window, &event_loop)?;
  gl_window.window().set_cursor_visible(false);

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

      if let Err(e) = lua::execute(ctx, "assets/scripts/init.lua") {
        log::error!("{}", e.to_string());
      }
    });
  }

  log::info!("configure open-gl");
  unsafe {
    gl::Enable(gl::DEPTH_TEST);
    gl::DepthFunc(gl::LEQUAL);
  }

  let mut start = Instant::now();
  event_loop.run(move |event, _, control_flow| {
    use glutin::event::{Event, WindowEvent, DeviceEvent, ElementState, StartCause, KeyboardInput, VirtualKeyCode};
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

    if start.elapsed().as_millis() > 20 {
      pump.run();

      let error = unsafe { gl::GetError() };

      if error != 0 {
        panic!("OPEN_GL ERROR {}", error);
      }

      unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
      }

      if let Some(scene) = &ctx.read().unwrap().scene {
        scene.read().unwrap().draw();
      }

      let error = unsafe { gl::GetError() };

      if error != 0 {
        panic!("OPEN_GL ERROR {}", error);
      }

      gl_window.swap_buffers().unwrap();
      start = Instant::now();
    }
  });
  //Ok(())
}
