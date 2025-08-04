fn main() {
  use bau::*;
  use std::{sync::Arc, thread, time::Duration};

  let resources = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
  let semaphore = Arc::new(Semaphore::new(resources));

  (0..semaphore.count() * 10)
    .map(|_| {
      let s = semaphore.clone();
      thread::spawn(move || {
        let res = s.wait().unwrap();

        // Critical section
        let _: i32 = dbg!(*res);
        thread::sleep(Duration::from_millis(rand::random_range(0..100)));

        // Call to signal upon exit
      })
    })
    .collect::<Vec<thread::JoinHandle<()>>>()
    .into_iter()
    .for_each(|h| h.join().unwrap());
}
