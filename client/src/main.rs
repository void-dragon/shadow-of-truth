mod methatron;
mod context;
mod lua;
mod tracer;
mod network;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let env = env_logger::Env::default().default_filter_or("debug");
  env_logger::Builder::from_env(env).init();

  let event_loop = glutin::event_loop::EventLoop::new();
  let window = glutin::window::WindowBuilder::new()
    .with_title("shadow-of-truth");
  let gl_window = glutin::ContextBuilder::new()
      .build_windowed(window, &event_loop)
      .unwrap();
  gl_window.window().set_cursor_visible(false);

  // It is essential to make the context current before calling `gl::load_with`.
  let gl_window = unsafe { gl_window.make_current() }.unwrap();

  // Load the OpenGL function pointers
  gl::load_with(|symbol| gl_window.get_proc_address(symbol));

  log::info!("init luajit");

  let ctx = context::new();
  let mut net = network::new();
  let pump = methatron::pump::get();

  net.establish_connection();

  {
    let ctx = ctx.clone();
    std::thread::spawn(move || {
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

  event_loop.run(move |event, _, control_flow| {
    use glutin::event::{Event, WindowEvent, DeviceEvent, ElementState};
    use glutin::event_loop::ControlFlow;
    *control_flow = ControlFlow::Wait;

    pump.run();

    let error = unsafe { gl::GetError() };

    if error != 0 {
      panic!("OPEN_GL ERROR {}", error);
    }

    match event {
      Event::LoopDestroyed => return,
      Event::WindowEvent { event, .. } => match event {
          WindowEvent::CloseRequested => {
              // Cleanup
              *control_flow = ControlFlow::Exit
          },
          _ => (),
      },
      Event::DeviceEvent { event, .. } => {
        match event {
          DeviceEvent::Key(key) => {
            if let Some(code) = key.virtual_keycode {
              let name = format!("{:?}", code);

              match key.state {
                ElementState::Pressed => {
                  ctx.write().unwrap().keys_down.insert(name);
                }
                ElementState::Released => {
                  ctx.write().unwrap().keys_down.remove(&name);
                }
              }
            }
          }
          DeviceEvent::MouseMotion{delta} => {
            let mut c = ctx.write().unwrap();
            c.mouse_position[0] += delta.0;
            c.mouse_position[1] += delta.1;
          }
          _ => {}
        }
      }
      Event::RedrawRequested(_) => { },
      _ => (),
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
  });

  Ok(())
}
