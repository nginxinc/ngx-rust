# Nginx Mixer Module

## Development Set up for Mac


### Install Rust 1.18.0

Current rust version 1.19.0 has issue with linking in Mac.  Use 1.18.0 until this issue is sorted out.

First install rust at:  https://www.rust-lang.org/en-US/install.html

Then switch to Rust 1.18.0:

`rustup install 1.18.0
rustup default 1.18.0`

### Install CLang for bindgen

Install Clang at: https://rust-lang-nursery.github.io/rust-bindgen/requirements.html


### set up source
make darwin-source


### build 
cargo build




