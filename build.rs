extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {

    let nginxDir = "./nginx";
    
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .clang_arg(format!("-I{}/src/core",nginxDir))
        .clang_arg(format!("-I{}/src/event",nginxDir))
        .clang_arg(format!("-I{}/src/event/modules",nginxDir))
        .clang_arg(format!("-I{}/src/os/unix",nginxDir))
        .clang_arg(format!("-I{}/objs",nginxDir))
        .clang_arg(format!("-I{}/src/http",nginxDir))
        .clang_arg(format!("-I{}/src/http/modules",nginxDir))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");


}
