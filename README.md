# ngx-rust

Module development kit for Nginx using Rust.  This crates does 2 things:

(1) Generates C bindings for using Nginx inside Rust module.
(2) Provide limited high level wrapper for Nginx C interface.


## Getting Started

Add the following dependency to your Cargo manifest...

```toml
[dependencies]
ngix-rust = "0.1.1"
```

Next, add this to your crate:

```rust
extern crate ngx-rust;
```

## to ensure bindings are formatted, run following cmd
```rustup component add rustfmt-preview```

## Limitation

Currently only Darwin (Mac OSX) and Linux platform are supported

