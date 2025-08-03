# `bau`

## A fun little signaling crate

```rust
fn main() {
  let resource = std::sync::Arc::new(11);
  let semaphore = bau::Semaphore::new(10);

  let r = resource.clone();
  let s = semaphore.clone();

  std::thread::spawn(move || {
    let _: i32 = semaphore
      .wait(|| {
        *resource + *resource // Critical section
      })
      .unwrap(); // Call to signal upon exit
  });

  std::thread::spawn(move || {
    let _: i32 = {
      let _guard = s.wait_guard();
      *r + *r // Critical section
    }; // Call to signal upon drop
  });

  println!("BAU BAU!");
}
```

> [!NOTE]
> No AI-generated code.
