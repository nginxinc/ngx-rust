[![Rust](https://github.com/nginxinc/ngx-rust/actions/workflows/ci.yaml/badge.svg)](https://github.com/nginxinc/ngx-rust/actions/workflows/ci.yaml)
[![crates.io](https://img.shields.io/crates/v/ngx.svg)](https://crates.io/crates/ngx)
[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![Community Support](https://badgen.net/badge/support/community/cyan?icon=awesome)](https://github.com/nginxinc/ngx-rust/discussions)


## Project status
This project is still a work in progress and not production ready.

# Description

This project provides Rust SDK interfaces to the [NGINX](https://nginx.com) proxy allowing the creation of NGINX
dynamic modules completely in Rust.

In short, this SDK allows writing NGINX modules using the Rust language.

## Build

NGINX modules can be built against a particular version of NGINX. The following environment variables can be used to specify a particular version of NGINX or an NGINX dependency:

* `ZLIB_VERSION` (default 1.3) -
* `PCRE2_VERSION` (default 10.42)
* `OPENSSL_VERSION` (default 3.0.7)
* `NGX_VERSION` (default 1.23.3) - NGINX OSS version
* `NGX_DEBUG` (default to false)-  if set to true, then will compile NGINX `--with-debug` option

For example, this is how you would compile the [examples](examples) using a specific version of NGINX and enabling
debugging:
```
NGX_DEBUG=true NGX_VERSION=1.23.0 cargo build --package=examples --examples --release
```

To build Linux-only modules, use the "linux" feature:
```
cargo build --package=examples --examples --features=linux --release
```

After compilation, the modules can be found in the path `target/release/examples/` ( with the `.so` file extension for
Linux or `.dylib` for MacOS).

Additionally, the folder  `.cache/nginx/{NGX_VERSION}/{OS}/` will contain the compiled version of NGINX used to build
the SDK. You can start NGINX directly from this directory if you want to test the module.

### Mac OS dependencies

In order to use the optional GNU make build process on MacOS, you will need to install additional tools. This can be
done via [homebrew](https://brew.sh/) with the following command:
```
brew install make openssl grep
```

Additionally, you may need to set up LLVM and clang. Typically, this is done as follows:

```
# make sure xcode tools are installed
xcode-select --install
# instal llvm
brew install --with-toolchain llvm
```

### Linux dependencies

See the [Dockerfile](Dockerfile) for dependencies as an example of required packages on Debian Linux.

### Build example

Example modules are available in [examples](examples) folder. You can use `cargo build --package=examples --examples` to build these examples. After building, you can find the `.so` or `.dylib` in the `target/debug` folder. Add `--features=linux` to build linux specific modules. **NOTE**: adding the "linux" feature on MacOS will cause a build failure.

For example (all examples plus linux specific):
`cargo build --package=examples --examples --features=linux`

### Docker

We provide a multistage [Dockerfile](Dockerfile):

    # build all dynamic modules examples and specify NGINX version to use
    docker buildx build --build-arg NGX_VERSION=1.23.3 -t ngx-rust .

    # start NGINX using [curl](examples/curl.conf) module example:
    docker run --rm -d  -p 8000:8000 ngx-rust nginx -c examples/curl.conf

    # test it - you should see 403 Forbidden
    curl http://127.0.0.1:8000 -v -H "user-agent: curl"


    # test it - you should see 404 Not Found
    curl http://127.0.0.1:8000 -v -H "user-agent: foo"

## Usage

A complete module example using the SDK can be found [here](examples/curl.rs). You can build it with
`cargo build --package=examples --example=curl` then set up NGINX to use it:

For example:
```nginx
daemon off;
master_process off;

# unix:
# load_module modules/libcurl.so;

# error_log logs/error.log debug;
error_log /dev/stdout debug;

working_directory /tmp/cores/;
worker_rlimit_core 500M;

events {
}

http {
    access_log /dev/stdout;
    server {
        listen 8000;
        server_name localhost;
        location / {
            alias /srv/http;
            # ... Other config stuff ...

            curl on;
        }
    }
}
```

## Support
This SDK is currently unstable. Right now, our primary goal is collect feedback and stabilize it be before
offering support. Feel free [contributing](CONTRIBUTING.md) by creating issues, PRs, or github discussions.

Currently, the only supported platforms are:
* Darwin (Mac OSX)
* Linux platform

## Roadmap
If you have ideas for releases in the future, please suggest them in the github discussions.

## Contributing

We welcome pull requests and issues!

Please refer to the [Contributing Guidelines](CONTRIBUTING.md) when doing a PR.

## Authors and acknowledgment
This project uses some great work from [dcoles/nginx-rs](https://github.com/dcoles/nginx-rs),
[arvancloud/nginx-rs](https://github.com/arvancloud/nginx-rs).

## License

All code in this repository is licensed under the
[Apache License v2 license](LICENSE.txt).
