# plumbum-rs

> simple shell command interaction mimicking python's plumbum library

> work in progress!

## Usage

So far only piping things works. Command generation and usage in general isn't very ergonomic at the moment.

```rust
#[macro_use] extern crate plumbum;

use plumbum::Local;
use plumbum::errors::*;

/// running from this crate root
pub fn main() {
    let local = Local::new();

    let mut cat = local.bin("cat").unwrap();
    cat.arg("Cargo.toml");
    let mut grep = local.bin("grep").unwrap();
    grep.args(&["walkdir"]);

    let out = pipe!(cat | grep);
    println!("{:?}", out);
}

```
