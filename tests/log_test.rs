use std::env;
use std::fs;
use std::io::Result;
use std::process::Command;
use std::process::Output;

const NGX_DEFAULT_VERSION: &str = "1.23.3";
const NGINX_BIN: &str = "sbin/nginx";
const NGINX_CONFIG: &str = "conf/nginx.conf";

/// harness to test nginx
pub struct Nginx {
    pub install_path: String,
}

impl Default for Nginx {
    /// create nginx with default
    fn default() -> Nginx {
        let path = env::current_dir().unwrap();
        let version = env::var("NGX_VERSION").unwrap_or_else(|_| NGX_DEFAULT_VERSION.to_string());
        let platform = format!("{}-{}", env::consts::OS, env::consts::ARCH);
        let ngx_path = format!(".cache/nginx/{}/{}", version, platform);
        let install_path = format!("{}/{}", path.display(), ngx_path);
        Nginx { install_path }
    }
}

impl Nginx {
    pub fn new(path: String) -> Nginx {
        Nginx { install_path: path }
    }

    /// get bin path to nginx instance
    pub fn bin_path(&mut self) -> String {
        format!("{}/{}", self.install_path, NGINX_BIN)
    }

    /// start nginx process with arguments
    pub fn cmd(&mut self, args: &[&str]) -> Result<Output> {
        let bin_path = self.bin_path();
        let result = Command::new(bin_path).args(args).output();

        match result {
            Err(e) => Err(e),

            Ok(output) => {
                println!("status: {}", output.status);
                println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
                println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                Ok(output)
            }
        }
    }

    /// complete stop the nginx binary
    pub fn stop(&mut self) -> Result<Output> {
        self.cmd(&["-s", "stop"])
    }

    /// start the nginx binary
    pub fn start(&mut self) -> Result<Output> {
        self.cmd(&[])
    }

    // make sure we stop existing nginx and start new master process
    // intentinally ignore failure in stop
    pub fn restart(&mut self) -> Result<Output> {
        let _ = self.stop();
        self.start()
    }

    // replace config with another config
    pub fn replace_config(&mut self, from: &str) -> Result<u64> {
        let config_path = format!("{}/{}", self.install_path, NGINX_CONFIG);
        println!("copying config from: {} to: {}", from, config_path); // replace with logging
        fs::copy(from, config_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    const TEST_NGINX_CONFIG: &str = "tests/nginx.conf";

    #[test]
    fn test() {
        let mut nginx = Nginx::default();

        let current_dir = env::current_dir().expect("Unable to get current directory");
        let test_config_path = current_dir.join(TEST_NGINX_CONFIG);

        assert!(
            test_config_path.exists(),
            "Config file not found: {}\nCurrent directory: {}",
            test_config_path.to_string_lossy(),
            current_dir.to_string_lossy()
        );

        nginx
            .replace_config(&test_config_path.to_string_lossy())
            .expect(format!("Unable to load config file: {}", test_config_path.to_string_lossy()).as_str());
        let output = nginx.restart().expect("Unable to restart NGINX");
        assert!(output.status.success());

        let output = nginx.stop().expect("Unable to stop NGINX");
        assert!(output.status.success());
    }
}
