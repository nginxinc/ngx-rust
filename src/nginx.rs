/**
 * harness to test nginx
 */

use std::process::Command;
use std::process::Output;
use std::io::Result;
use std::env;
use std::fs;

const NGINX_INSTALL_PATH: &str = "nginx/install";
const NGINX_BIN: &str = "sbin/nginx";
const NGINX_CONFIG: &str = "conf/nginx.conf";

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
        let install_path = format!("{}/{}",path.display(),NGINX_INSTALL_PATH);
        Nginx { install_path: install_path }
    }


    // get bin path
    pub fn bin_path(&mut self) -> String  {
        format!("{}/{}",self.install_path,NGINX_BIN)
    }


    pub fn cmd(&mut self, args: &[&str] )  -> Result<Output> {
        let bin_path = self.bin_path();
        let result =  Command::new(&bin_path)
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
    pub fn replace_config(&mut self, from: &str) -> Result<u64> {
        let config_path = format!("{}/{}",self.install_path,NGINX_CONFIG);
        println!("copying config from: {} to: {}",from,config_path); // replace with logging
        fs::copy(from , config_path)
    }


}








