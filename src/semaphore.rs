use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};

#[derive(Debug)]
struct InnerSemaphore {
  count: Mutex<usize>,
  var: Condvar,
}

#[derive(Debug, Clone)]
pub struct Semaphore {
  inner: Arc<InnerSemaphore>,
}

impl Semaphore {
  pub fn new(count: usize) -> Self {
    Self {
      inner: Arc::new(InnerSemaphore {
        count: Mutex::new(count),
        var: Condvar::new(),
      }),
    }
  }

  pub fn signal(&self) -> Result<(), PoisonError<MutexGuard<'_, usize>>> {
    let mut count = self.inner.count.lock()?;
    *count += 1;
    self.inner.var.notify_all();
    Ok(())
  }

  pub fn wait(&self) -> Result<(), PoisonError<MutexGuard<'_, usize>>> {
    let var = &self.inner.var;
    let mut count = var.wait_while(self.inner.count.lock().unwrap(), |count| *count == 0)?;
    *count = (*count).saturating_sub(1);
    Ok(())
  }
}
