use std::{
  cell::UnsafeCell,
  collections::HashMap,
  ops::{Deref, DerefMut},
  sync::{Condvar, Mutex, MutexGuard, PoisonError},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum State {
  Acquired,
  #[default]
  Free,
}

#[derive(Debug)]
pub struct Semaphore<const RESOURCE_COUNT: usize, T> {
  states: UnsafeCell<HashMap<usize, State>>,
  data: UnsafeCell<[T; RESOURCE_COUNT]>,
  count: Mutex<usize>,
  var: Condvar,
}

unsafe impl<const RESOURCE_COUNT: usize, T: Send> Send for Semaphore<RESOURCE_COUNT, T> {}
unsafe impl<const RESOURCE_COUNT: usize, T: Sync> Sync for Semaphore<RESOURCE_COUNT, T> {}

impl<const RESOURCE_COUNT: usize, T> Semaphore<RESOURCE_COUNT, T> {
  pub fn new(resources: [T; RESOURCE_COUNT]) -> Self {
    Self {
      states: UnsafeCell::new((0..RESOURCE_COUNT).map(|x| (x, State::Free)).collect()),
      data: UnsafeCell::new(resources),
      count: Mutex::new(RESOURCE_COUNT),
      var: Condvar::new(),
    }
  }

  pub fn count(&self) -> usize {
    unsafe { &*self.data.get() }.len()
  }

  pub fn wait(&self) -> Result<SemaphoreGuard<RESOURCE_COUNT, T>, PoisonError<MutexGuard<'_, usize>>> {
    let var = &self.var;
    let mut count = var.wait_while(self.count.lock()?, |count| *count == 0)?;
    *count = (*count).saturating_sub(1);
    let data = unsafe { &mut *self.data.get() };
    let states = unsafe { &mut *self.states.get() };
    let index = *states.iter().find(|e| e.1 == &State::Free).unwrap().0;
    *states.get_mut(&index).unwrap() = State::Acquired;
    let next = &mut data[index];
    Ok(SemaphoreGuard {
      semaphore: self,
      resource: next,
      index,
    })
  }

  fn signal(&self, index: usize) -> Result<(), PoisonError<MutexGuard<'_, usize>>> {
    let mut count = self.count.lock()?;
    *count = (*count).saturating_add(1);
    let states = unsafe { &mut *self.states.get() };
    *states.get_mut(&index).unwrap() = State::Free;
    self.var.notify_one();
    Ok(())
  }
}

#[must_use = "Semaphore will be signaled when the guard is dropped"]
pub struct SemaphoreGuard<'s, const RESOURCE_COUNT: usize, T> {
  semaphore: &'s Semaphore<RESOURCE_COUNT, T>,
  resource: &'s mut T,
  index: usize,
}

impl<'s, const RESOURCE_COUNT: usize, T> Drop for SemaphoreGuard<'s, RESOURCE_COUNT, T> {
  fn drop(&mut self) {
    self.semaphore.signal(self.index).unwrap();
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
