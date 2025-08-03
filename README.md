# `bau`

## A fun little signaling crate

```rust
fn main() {
  let resource = std::sync::Arc::new(11);
  let semaphore = bau::Semaphore::new(10);

  let r = resource.clone();
  let s = semaphore.clone();

  std::thread::spawn(move || {
    let _res: std::sync::Arc<i32> = semaphore // Entry
      .wait(|| {
        resource.clone() // Critical section
      })
      .unwrap(); // Call to signal upon exit
  });

  std::thread::spawn(move || {
    {
      let _guard = s.wait_guard(); // Entry
      let _res = r;
      println!("I'm not fuwawa!"); // Critical section
    } // Call to signal upon drop
  });

  println!("BAU BAU!");
}
```

> [!NOTE]
> No AI-generated code.
