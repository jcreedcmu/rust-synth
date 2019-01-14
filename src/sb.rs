use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

fn wait(n: u64) {
  sleep(Duration::from_millis(n));
}

pub fn dance() {
  let k = Arc::new(Mutex::new(42));

  let z = k.clone();

  thread::spawn(move || {
    println!("Alice: Hello");
    {
      println!("Alice: locking");
      let mut x = z.lock().unwrap();
      let xread = *x;
      println!("Alice saw, {}", xread);
      wait(1000);
      let xwrite = xread + 43;
      println!("Alice writing, {}", xwrite);
      *x = xwrite;
      println!("Alice: Unlocking I hope");
    }
    wait(25000);
  });

  let z2 = k.clone();
  thread::spawn(move || {
    wait(100);
    {
      println!("Bob attempting to read");
      let mut x = z2.lock().unwrap();
      let xread = *x;
      println!("Bob saw, {}", xread);
      wait(1000);
      let xwrite = xread + 43;
      println!("Bob writing, {}", xwrite);
      *x = xwrite;
      println!("Bob: Unlocking I hope");
    }
  });
}
