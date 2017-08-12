extern crate bindgen;

use std::env;
use std::path::PathBuf;

#[cfg(target_os = "macos")]
const nginx_dir: &str  = "./nginx/nginx-darwin";

#[cfg(target_os = "linux")]
const nginx_dir: &str  = "./nginx/nginx-linux";

fn main() {


    
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .clang_arg(format!("-I{}/src/core",nginx_dir))
        .clang_arg(format!("-I{}/src/event",nginx_dir))
        .clang_arg(format!("-I{}/src/event/modules",nginx_dir))
        .clang_arg(format!("-I{}/src/os/unix",nginx_dir))
        .clang_arg(format!("-I{}/objs",nginx_dir))
        .clang_arg(format!("-I{}/src/http",nginx_dir))
        .clang_arg(format!("-I{}/src/http/modules",nginx_dir))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");


}
