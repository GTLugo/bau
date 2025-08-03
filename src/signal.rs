use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};

#[derive(Debug)]
struct InnerSignal {
  flag: Mutex<bool>,
  var: Condvar,
}

#[derive(Debug, Clone)]
pub struct Signal {
  inner: Arc<InnerSignal>,
}

impl Default for Signal {
  fn default() -> Self {
    Self::new()
  }
}

impl Signal {
  pub fn new() -> Self {
    Self {
      inner: Arc::new(InnerSignal {
        flag: Mutex::new(true),
        var: Condvar::new(),
      }),
    }
  }

  pub fn signal(&self) -> Result<(), PoisonError<MutexGuard<'_, bool>>> {
    let mut waiting = self.inner.flag.lock()?;
    *waiting = false;
    self.inner.var.notify_all();
    Ok(())
  }

  pub fn wait(&self) -> Result<(), PoisonError<MutexGuard<'_, bool>>> {
    let var = &self.inner.var;
    let mut waiting = var.wait_while(self.inner.flag.lock().unwrap(), |waiting| *waiting)?;
    *waiting = true;
    Ok(())
  }
}
