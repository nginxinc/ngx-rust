# ngx-rust

Module development kit for Nginx using Rust

## Requirement

this crate use bazel to install nginx source and generate bindings.  Please follow https://docs.bazel.build/versions/master/install.html.

## Usage

First, add this to your 'Cargo.toml';

```toml
[dependencies]
ngix-rust = "0.1"
```

Next, add this to your crate:

```rust
extern crate ngx-rust;
```

## Limitation

Currently only Darwin (Mac OSX) and Linux platform are supported




