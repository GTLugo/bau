fn main() {
  let resource = 11;
  let semaphore = bau::Semaphore::new(10, resource);

  std::thread::spawn(move || {
    let resource = semaphore.wait().unwrap();
    let _: i32 = *resource + *resource; // Critical section
    // Call to signal upon exit
  });

  println!("BAU BAU!");
}
