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

  pub fn wait<T>(&self, f: impl FnOnce() -> T) -> Result<T, PoisonError<MutexGuard<'_, usize>>> {
    let _guard = self.wait_guard()?;
    Ok(f())
  }

  pub fn wait_guard(&self) -> Result<SemaphoreGuard, PoisonError<MutexGuard<'_, usize>>> {
    let var = &self.inner.var;
    let mut count = var.wait_while(self.inner.count.lock().unwrap(), |count| *count == 0)?;
    *count = (*count).saturating_sub(1);
    Ok(SemaphoreGuard { semaphore: self })
  }

  fn signal(&self) -> Result<(), PoisonError<MutexGuard<'_, usize>>> {
    let mut count = self.inner.count.lock()?;
    *count += 1;
    self.inner.var.notify_all();
    Ok(())
  }
}

#[must_use = "Semaphore will be signaled when the guard is dropped"]
pub struct SemaphoreGuard<'s> {
  semaphore: &'s Semaphore,
}

impl<'s> Drop for SemaphoreGuard<'s> {
  fn drop(&mut self) {
    self.semaphore.signal().unwrap();
  }
}
