/**
 * harness to test nginx
 */

use std::process::Command;
use std::process::Output;
use std::io::Result;
use std::env;
use std::fs;

const NGINX_BIN: &str = "nginx/install/sbin/nginx";

pub struct Nginx  {

    pub install_path: String            // install path
}


impl Nginx  {

    pub fn new(path: String) -> Nginx  {
        Nginx { install_path: path }
    }

    // create nginx with default
    pub fn default() -> Nginx  {
        let path = env::current_dir().unwrap();
        let nginx_bin_path = format!("{}/{}",path.display(),NGINX_BIN);
        Nginx { install_path: nginx_bin_path}
    }


    pub fn cmd(&mut self, args: &[&str] )  -> Result<Output> {
        let result =  Command::new(&self.install_path)
            .args(args)
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

    // complete stop the nginx binary
    pub fn stop(&mut self)  -> Result<Output> {
        self.cmd(&["-s","stop"])
    }

    // start the nginx binary
    pub fn start(&mut self) -> Result<Output> {
        self.cmd(&[])
    }


    // make sure we stop existing nginx and start new master process
    // intentinally ignore failure in stop
    pub fn restart(&mut self) -> Result<Output> {

        self.stop();
        self.start()
    }

    // replace config with another config
    pub fn replace_config(&mut self, path: &str) {

    }


}





