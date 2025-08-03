use std::{
  cell::UnsafeCell,
  ops::{Deref, DerefMut},
  sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError},
};

#[derive(Debug)]
struct InnerSemaphore<T> {
  data: UnsafeCell<T>,
  count: Mutex<usize>,
  var: Condvar,
}

#[derive(Debug, Clone)]
pub struct Semaphore<T> {
  inner: Arc<InnerSemaphore<T>>,
}

unsafe impl<T: Send> Send for Semaphore<T> {}
unsafe impl<T: Sync> Sync for Semaphore<T> {}

impl<T> Semaphore<T> {
  pub fn new(count: usize, data: T) -> Self {
    Self {
      inner: Arc::new(InnerSemaphore {
        data: UnsafeCell::new(data),
        count: Mutex::new(count),
        var: Condvar::new(),
      }),
    }
  }

  pub fn wait(&self) -> Result<SemaphoreGuard<T>, PoisonError<MutexGuard<'_, usize>>> {
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
pub struct SemaphoreGuard<'s, T> {
  semaphore: &'s Semaphore<T>,
}

impl<'s, T> Drop for SemaphoreGuard<'s, T> {
  fn drop(&mut self) {
    self.semaphore.signal().unwrap();
  }
}

impl<'s, T> Deref for SemaphoreGuard<'s, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.semaphore.inner.data.get() }
  }
}

impl<'s, T> DerefMut for SemaphoreGuard<'s, T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.semaphore.inner.data.get() }
  }
}
