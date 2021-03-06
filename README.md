c_str [![Build Status](https://api.travis-ci.org/GuillaumeGomez/c_str-rs.png?branch=master)](https://travis-ci.org/GuillaumeGomez/c_str-rs)
=====

Old rust c_str module. It provides the ToCStr and FromCStr traits. It works just like the old one:

```Rust
extern crate libc;
extern crate c_str;

use c_str::{FromCStr, ToCStr};

fn some_func(cstr: *const libc::c_char) {
    let s : String = FromCStr::from_c_str(cstr);

    println!("converted from c string: {}", s);
}

fn some_other_func(rstr: &str) {
    unsafe {
        rstr.with_c_str(|s| {
            some_c_func(s)
        })
    }
}
```

Equivalent in Rust
==================

Here is the equivalent in pure Rust:

```Rust
fn from_c_func(cstr: *const libc::c_char) -> String {
    FromCStr::from_c_str(tmp)
    // equivalent in rust:
    String::from_utf8_lossy(::std::ffi::c_str_to_bytes(&tmp).to_string())
}

fn main() {
    let s = "hello";

    s.with_c_str(|cstr| {
        from_c_func(cstr)
    });
    // equivalent in rust:
    let cstring = CString::from_slice(s.as_bytes());

    from_c_func(cstring.as_ptr());
}
```

Usage
=====

You can use it directly by adding this line to your `Cargo.toml` file:

```Rust
[dependencies]
c_str = "^1.0.0"
```

Here's is the [crates.io](https://crates.io/crates/c_str) page for `c_str`.

License
=======

This project is under the MIT and Apache 2.0 licenses. Please look at the license files for more information.
