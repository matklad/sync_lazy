[![Build Status](https://travis-ci.org/matklad/sync_lazy.svg?branch=master)](https://travis-ci.org/matklad/sync_lazy)
[![Crates.io](https://img.shields.io/crates/v/sync_lazy.svg)](https://crates.io/crates/sync_lazy)
[![API reference](https://docs.rs/sync_lazy/badge.svg)](https://docs.rs/sync_lazy/)

# Overview

A thread safe lazy values for Rust.


```Rust
#[macro_use]
extern crate sync_lazy;

use std::collections::HashMap;
use sync_lazy::Lazy;

static GLOBAL: Lazy<HashMap<i32, String>> = sync_lazy! {
    println!("initializing global");
    let mut m = HashMap::new();
    m.insert(13, "Spica".to_string());
    m.insert(74, "Hoyten".to_string());
    m
};

fn main() {
    println!("ready");
    let xs = vec![1, 2, 3];
    let local = Lazy::new(move || {
        println!("initializing local");
        xs.into_iter().sum::<i32>()
    });

    ::std::thread::spawn(|| {
        println!("{:?}", GLOBAL.get(&13));
    }).join().unwrap();
    println!("{:?}", GLOBAL.get(&74));
    println!("{}", Lazy::force(&local));
    println!("{}", Lazy::force(&local));

    // Prints:
    //   ready
    //   initializing global
    //   Some("Spica")
    //   Some("Hoyten")
    //   initializing local
    //   6
    //   6
}
```
