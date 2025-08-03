fn main() {
  let semaphore = bau::Semaphore::new(10);
  let s = semaphore.clone();

  std::thread::spawn(move || {
    semaphore // Entry
      .wait(|| {
        println!("I'm not a chihuahua!"); // Critical section
      })
      .unwrap(); // Call to signal upon exit
  });

  std::thread::spawn(move || {
    {
      let _guard = s.wait_guard(); // Entry
      println!("I'm not fuwawa!"); // Critical section
    } // Call to signal upon drop
  });

  println!("BAU BAU!");
}
