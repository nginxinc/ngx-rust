extern crate bindgen;

#[cfg(target_os = "macos")]
const NGIX_DIR: &str  = "./nginx/nginx-darwin";

#[cfg(target_os = "linux")]
const NGIX_DIR: &str  = "./nginx/nginx-linux";

fn main() {

    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .layout_tests(false)
        .clang_arg(format!("-I{}/src/core",NGIX_DIR))
        .clang_arg(format!("-I{}/src/event",NGIX_DIR))
        .clang_arg(format!("-I{}/src/event/modules",NGIX_DIR))
        .clang_arg(format!("-I{}/src/os/unix",NGIX_DIR))
        .clang_arg(format!("-I{}/objs",NGIX_DIR))
        .clang_arg(format!("-I{}/src/http",NGIX_DIR))
        .clang_arg(format!("-I{}/src/http/modules",NGIX_DIR))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");

}
