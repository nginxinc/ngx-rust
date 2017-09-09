extern crate bindgen;

use std::process::Command;
use std::process::Output;
use std::env;
use std::io::Result;

#[cfg(target_os = "macos")]
const NGIX_DIR: &str  = "./nginx/nginx-darwin";

// perform make with argument
fn make(arg: &str) -> Result<Output> {
    let current_path = env::current_dir().unwrap();
    let path_name = format!("{}",current_path.display());
    println!("executing make command at {}",path_name);
    let result =  Command::new("/usr/bin/make")
        .args(&[arg])
        .current_dir(path_name)
        .output();

    match result  {
        Err(e)  =>  {
            return Err(e);
        },

        Ok(output) => {
            println!("status: {}", output.status);
            println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            return Ok(output);
        }
    }
}



#[cfg(target_os = "macos")]
fn configure() -> Result<Output> {
    make("darwin-setup")
}



#[cfg(target_os = "linux")]
const NGIX_DIR: &str  = "./nginx/nginx-linux";

fn generate_binding() {
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

fn main() {

    configure();
    generate_binding();

}
