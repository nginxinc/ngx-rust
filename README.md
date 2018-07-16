# Rust for NGINX

Rust bindings and wrappers for NGINX.  Can be used for building dynamic modules and hacking NGINX using rust.

## Production Status

This version is proof of concept.  It has enough binding for building modules for [nginmesh](https://github.com/nginxinc/nginmesh).  

You still need to write C stub code to build the complete module.  Please wait for next version which will remove this restriction.

## Getting Started

Add the following dependency to your Cargo manifest...

```toml
[dependencies]
ngx_rust = "0.1.1"
```

Next, add this to your crate:

```rust
extern crate ngx_rust;
```

## Building module Example

Please see [istio mixer module](https://github.com/nginxinc/ngx-istio-mixer) for full example.
Currently, it requires much machinery to build the module. 

## Roadmap

Please see [roadmap](https://github.com/nginxinc/ngx-rust/wiki) for future plans.  



## Limitation

Only supports these platforms:
- Darwin (Mac OSX)
- Linux platform

