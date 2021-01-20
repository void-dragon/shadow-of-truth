use std::sync::{
  Arc,
  Mutex,
  Condvar,
};
use std::collections::VecDeque;


static mut PUMP: Option<Pump> = None;

pub fn get() -> Pump {
  unsafe {
    if let Some(ref p) = PUMP {
      p.clone()
    }
    else {
      let p = Pump {
        queue: Arc::new(Mutex::new(VecDeque::new())),
      };
      PUMP = Some(p.clone());
      p
    }
  }
}

#[derive(Clone)]
pub struct Pump {
  queue: Arc<Mutex<VecDeque<Arc<dyn FnMut() + Send + Sync + 'static>>>>,
}

impl Pump {
  pub fn exec<T, E: Send + Sync + 'static>(&self, mut func: T) -> E
  where 
    T: FnMut() -> E + Send + Sync + 'static, 
  {
    let pair = Arc::new((Mutex::new(None), Condvar::new()));
    let pair2 = pair.clone();

    self.queue.lock().unwrap().push_back(Arc::new(move || {
      let (mtx, var) = &*pair2;
      let mut result = mtx.lock().unwrap();
      *result = Some(func());
      var.notify_one();
    }));

    let (mtx, var) = &*pair;
    let mut result = mtx.lock().unwrap();
    while result.is_none() {
      result = var.wait(result).unwrap();
    }

    result.take().unwrap()
  }

  pub fn run(&self) {
    let callbacks: Vec<Arc<dyn FnMut() + Send + Sync + 'static>> = self.queue.lock().unwrap().drain(..).collect();

    for mut cb in callbacks {
      if let Some(cb) = Arc::get_mut(&mut cb) {
        cb();
      }
    }
  }
}