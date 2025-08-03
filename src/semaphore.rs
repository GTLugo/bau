use std::{
  cell::UnsafeCell,
  ops::{Deref, DerefMut},
  sync::{Condvar, Mutex, MutexGuard, PoisonError},
};

#[derive(Debug)]
pub struct Semaphore<const RESOURCE_COUNT: usize, T> {
  data: UnsafeCell<[T; RESOURCE_COUNT]>,
  count: Mutex<usize>,
  var: Condvar,
}

unsafe impl<const RESOURCE_COUNT: usize, T: Send> Send for Semaphore<RESOURCE_COUNT, T> {}
unsafe impl<const RESOURCE_COUNT: usize, T: Sync> Sync for Semaphore<RESOURCE_COUNT, T> {}

impl<const RESOURCE_COUNT: usize, T> Semaphore<RESOURCE_COUNT, T> {
  pub fn new(resources: [T; RESOURCE_COUNT]) -> Self {
    Self {
      data: UnsafeCell::new(resources),
      count: Mutex::new(RESOURCE_COUNT),
      var: Condvar::new(),
    }
  }

  pub fn wait(&self) -> Result<SemaphoreGuard<RESOURCE_COUNT, T>, PoisonError<MutexGuard<'_, usize>>> {
    let var = &self.var;
    let mut count = var.wait_while(self.count.lock().unwrap(), |count| *count == 0)?;
    *count = (*count).saturating_sub(1);
    let resources = unsafe { &mut *self.data.get() };
    let next = &mut resources[*count];
    Ok(SemaphoreGuard {
      semaphore: self,
      resource: next,
    })
  }

  fn signal(&self) -> Result<(), PoisonError<MutexGuard<'_, usize>>> {
    let mut count = self.count.lock()?;
    *count = (*count).saturating_add(1);
    self.var.notify_all();
    Ok(())
  }
}

#[must_use = "Semaphore will be signaled when the guard is dropped"]
pub struct SemaphoreGuard<'s, const RESOURCE_COUNT: usize, T> {
  semaphore: &'s Semaphore<RESOURCE_COUNT, T>,
  resource: &'s mut T,
}

impl<'s, const RESOURCE_COUNT: usize, T> Drop for SemaphoreGuard<'s, RESOURCE_COUNT, T> {
  fn drop(&mut self) {
    self.semaphore.signal().unwrap();
  }
}

impl<'s, const RESOURCE_COUNT: usize, T> Deref for SemaphoreGuard<'s, RESOURCE_COUNT, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    self.resource
  }
}

impl<'s, const RESOURCE_COUNT: usize, T> DerefMut for SemaphoreGuard<'s, RESOURCE_COUNT, T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.resource
  }
}
