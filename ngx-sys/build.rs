extern crate bindgen;

use std::process::Command;
use std::process::Output;
use std::env;
use std::path::PathBuf;
use std::io::Result;
use std::vec::Vec;

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

// nginx source directory
fn nginx_dir() -> String  {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    format!("{}/nginx",out_dir.display())
}



// copy nginx to out directory
fn copy_nginx() -> Result<Output> {
    let current_path = env::current_dir().unwrap();
    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let dest_str = dst.to_str().unwrap();
    let mut args: Vec<&str> = vec!["-r","nginx",dest_str];
    let result =  Command::new("cp")
        .args(&args)
        .current_dir(env::current_dir().unwrap())
        .output();

    println!("copying nginx to: {}",dest_str);
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

fn configure() -> Result<Output> {
    let nginx_path_name = nginx_dir();
    println!("nginx auto config at {}",nginx_path_name);
    let mut args: Vec<&str> = Vec::new();
    args.push("auto/configure");
    
    let features = ngix_features();
    
    for feature in features {
        args.push(&feature)
    }
        
    let result =  Command::new("bash")
        .args(&args)
        .current_dir(nginx_path_name)
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
    let nginx_path_name = nginx_dir();
    let builder = bindgen::Builder::default()
    // The input header we would like to generate
    // bindings for.
    .header("wrapper.h")
    .layout_tests(false)
    .rustfmt_bindings(true)
    .clang_arg(format!("-I{}/src/core",nginx_path_name))
    .clang_arg(format!("-I{}/src/event",nginx_path_name))
    .clang_arg(format!("-I{}/src/event/modules",nginx_path_name))
    .clang_arg(format!("-I{}/src/os/unix",nginx_path_name))
    .clang_arg(format!("-I{}/objs",nginx_path_name))
    .clang_arg(format!("-I{}/src/http",nginx_path_name))
    .clang_arg(format!("-I{}/src/http/modules",nginx_path_name))
    // Finish the builder and generate the bindings.
    .generate()
    // Unwrap the Result and panic on failure.
    .expect("Unable to generate bindings");

    builder
    .write_to_file("src/bindings.rs")
    .expect("Couldn't write bindings!");
}

fn main() {

    copy_nginx();
    configure();
    generate_binding();

}
