# genawaiter

This crate lets you use generators on stable Rust. Instead of using `yield`, which won't be stabilized anytime soon, you use `async`/`await`, which is stable today:

```rust
use genawaiter::rc::{Co, Gen};

async fn odd_numbers_less_than_ten(co: Co<i32>) {
    let mut n = 1;
    while n < 10 {
        co.yield_(n).await;
        n += 2;
    }
}

for n in Gen::new(odd_numbers_less_than_ten) {
    println!("{}", n);
}
```

Result:

```text
1
3
5
7
9
```

See the docs for more.
