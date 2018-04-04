extern crate bindgen;

use std::process::Command;
use std::process::Output;
use std::env;
use std::io::Result;
use std::vec::Vec;

const NGIX_DIR: &str  = "./nginx/";

// return all nginx features
fn ngix_features() -> Vec<&'static str> {
    let mut features: Vec<&'static str> = Vec::new();
    if cfg!(feature = "with-compat") {
        features.push("--with-compat");
    }
    if cfg!(feature = "with-threads") {
        features.push("--with-threads");
    }
    if cfg!(feature = "with-http_addition_module") {
        features.push("--with-http_addition_module");
    }      
    if cfg!(feature = "with-http_auth_request_module") {
        features.push("--with-http_auth_request_module");
    }
    if cfg!(feature = "with-http_gunzip_module") {
        features.push("--with-http_gunzip_module");
    }
    if cfg!(feature = "with-http_auth_request_module") {
        features.push("--with-http_auth_request_module");
    }
    if cfg!(feature = "with-http_gzip_static_module") {
        features.push("--with-http_gzip_static_module");
    }
    if cfg!(feature = "with-http_random_index_module") {
        features.push("--with-http_random_index_module");
    }
    if cfg!(feature = "with-http_realip_module") {
        features.push("--with-http_realip_module");
    }  
    if cfg!(feature = "with-http_secure_link_module") {
        features.push("--with-http_secure_link_module");
    }    
    if cfg!(feature = "with-http_slice_module") {
        features.push("--with-http_slice_module");
    }
    if cfg!(feature = "with-http_stub_status_module") {
        features.push("--with-http_stub_status_module");
    } 
    if cfg!(feature = "with-http_sub_module") {
        features.push("--with-http_sub_module");
    }  
    if cfg!(feature = "with-stream") {
        features.push("--with-stream");
    }  
    if cfg!(feature = "with-stream_realip_module") {
        features.push("--with-stream_realip_module");
    }
    if cfg!(feature = "with-stream_ssl_preread_module") {
        features.push("--with-stream_ssl_preread_module");
    }  
    
    if cfg!(all(feature = "with-file-aio", target_os="linux")) {
        features.push("--with-file-aio");
    }
    if cfg!(all(feature = "with-file-aio", target_os="linux"))  {
        features.push("--with-http_ssl_module");
    }
    if cfg!(all(feature = "with-file-aio", target_os="linux"))  {
        features.push("--with-stream_ssl_module");
    } 


    if cfg!(target_os="linux")  {
        features.push("--with-cc-opt='-g -fstack-protector-strong -Wformat -Werror=format-security -Wp,-D_FORTIFY_SOURCE=2 -fPIC'");
        features.push("--with-ld-opt='-Wl,-Bsymbolic-functions -Wl,-z,relro -Wl,-z,now -Wl,--as-needed -pie'");
    } 
    

    println!("configuring nginx: {:?}",features);

    features
    
}

fn configure() -> Result<Output> {
    let current_path = env::current_dir().unwrap();
    let path_name = format!("{}/nginx",current_path.display());
    println!("executing make command at {}",path_name);
    let mut args: Vec<&str> = Vec::new();
    args.push("auto/configure");
    
    let features = ngix_features();
    
    for feature in features {
        args.push(&feature)
    }
        
    let result =  Command::new("bash")
        .args(&args)
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
