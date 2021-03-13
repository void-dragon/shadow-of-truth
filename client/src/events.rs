pub enum Events {
    Connected,
    Disconnected,
    KeyPressed(String),
    KeyReleased(String),
    MouseWheel(f32),
}

#[derive(Clone)]
pub struct EventPump {
  pub sender: crossbeam_channel::Sender<Events>, 
  pub receiver: crossbeam_channel::Receiver<Events>,
}

static mut EVENT_PUMP: Option<EventPump> = None;

pub fn get() -> EventPump {
  unsafe {
    if let Some(ref p) = EVENT_PUMP {
      p.clone()
    }
    else {
      let (s, r) = crossbeam_channel::bounded::<Events>(100);
      let p = EventPump {sender: s, receiver: r};
      EVENT_PUMP = Some(p.clone());
      p
    }
  }
}