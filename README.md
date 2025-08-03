# `bau`

## A fun little signaling crate

```rust
fn main() {
  use std::sync::*;
  use bau::*;

  let resource = 11;
  let semaphore = Arc::new(Semaphore::new([resource; 10]));

  std::thread::spawn(move || {
    let resource = semaphore.wait().unwrap();
    let _: i32 = *resource + *resource; // Critical section
    // Call to signal upon exit
  });

  println!("BAU BAU!");
}
```

> [!NOTE]
> No AI-generated code.
