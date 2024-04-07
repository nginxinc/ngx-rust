use std::ffi::OsStr;
use std::fs;
use std::fs::read_dir;
use std::io::Result;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

const NGINX_PREFIX: &str = nginx_sys::metadata::NGINX_INSTALL_DIR;

const NGINX_SBIN_SUFFIX: &str = "sbin/nginx";
const NGINX_MODULES_SUFFIX: &str = "modules";
const NGINX_CONF_SUFFIX: &str = "conf/nginx.conf";
const NGINX_CONF_PREFIX_SUFFIX: &str = "conf";
const NGINX_ERROR_LOG_SUFFIX: &str = "logs/error.log";
const NGINX_PID_SUFFIX: &str = "logs/nginx.pid";
const NGINX_LOCK_SUFFIX: &str = "logs/nginx.lock";

const NGINX_HTTP_LOG_SUFFIX: &str = "logs/access.log";
const NGINX_HTTP_CLIENT_BODY_SUFFIX: &str = "client_body_temp";
const NGINX_HTTP_PROXY_TEMP_SUFFIX: &str = "proxy_temp";
const NGINX_HTTP_FASTCGI_TEMP_SUFFIX: &str = "fastcgi_temp";
const NGINX_HTTP_UWSGI_TEMP_SUFFIX: &str = "uwsgi_temp";
const NGINX_HTTP_SCGI_TEMP_SUFFIX: &str = "scgi_temp";

/// harness to test nginx
#[allow(dead_code)]
pub struct Nginx {
    // these paths have options to change them from default paths (in prefix dir)
    // most of them are not used, but keep them for future uses
    prefix: PathBuf,
    sbin_path: PathBuf,
    modules_path: PathBuf,
    conf_path: PathBuf,
    conf_prefix: PathBuf,
    error_log_path: PathBuf,
    pid_path: PathBuf,
    lock_path: PathBuf,
    http_log_path: PathBuf,
    http_client_body_temp_path: PathBuf,
    http_proxy_temp_path: PathBuf,
    http_fastcgi_temp_path: PathBuf,
    http_uwsgi_temp_path: PathBuf,
    http_scgi_temp_path: PathBuf,
}

impl Default for Nginx {
    fn default() -> Nginx {
        Self::new_with_prefix(NGINX_PREFIX.into())
    }
}

impl Nginx {
    /// create nginx with prefix only
    pub fn new_with_prefix(prefix: PathBuf) -> Nginx {
        Nginx {
            sbin_path: prefix.join(NGINX_SBIN_SUFFIX),
            modules_path: prefix.join(NGINX_MODULES_SUFFIX),
            conf_path: prefix.join(NGINX_CONF_SUFFIX),
            conf_prefix: prefix.join(NGINX_CONF_PREFIX_SUFFIX),
            error_log_path: prefix.join(NGINX_ERROR_LOG_SUFFIX),
            pid_path: prefix.join(NGINX_PID_SUFFIX),
            lock_path: prefix.join(NGINX_LOCK_SUFFIX),
            http_log_path: prefix.join(NGINX_HTTP_LOG_SUFFIX),
            http_client_body_temp_path: prefix.join(NGINX_HTTP_CLIENT_BODY_SUFFIX),
            http_proxy_temp_path: prefix.join(NGINX_HTTP_PROXY_TEMP_SUFFIX),
            http_fastcgi_temp_path: prefix.join(NGINX_HTTP_FASTCGI_TEMP_SUFFIX),
            http_uwsgi_temp_path: prefix.join(NGINX_HTTP_UWSGI_TEMP_SUFFIX),
            http_scgi_temp_path: prefix.join(NGINX_HTTP_SCGI_TEMP_SUFFIX),
            prefix,
        }
    }

    /// execute nginx process with arguments
    pub fn cmd(&mut self, args: &[&str]) -> Result<Output> {
        let result = Command::new(&self.sbin_path).args(args).output();

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

    /// make sure we stop existing nginx and start new master process
    /// intentinally ignore failure in stop
    pub fn restart(&mut self) -> Result<Output> {
        let _ = self.stop();
        self.start()
    }

    /// replace config with another config
    pub fn copy_config(&mut self, conf_path_from: &Path) -> Result<u64> {
        let conf_path_to = self
            .conf_prefix
            .join(conf_path_from.file_name().unwrap_or(OsStr::new("unknown_conf")));
        println!(
            "copying config from: {} to: {}",
            conf_path_to.display(),
            conf_path_from.display()
        ); // replace with logging
        fs::copy(conf_path_from, conf_path_to)
    }
    /// create config from &str
    pub fn create_config_from_str(&mut self, conf_suffix: &str, conf_content: &str) -> Result<()> {
        let conf_path_to = self.conf_prefix.join(conf_suffix);
        println!(
            "creating config to: {} content: {}",
            conf_path_to.display(),
            conf_content
        ); // replace with logging
        fs::write(conf_path_to, conf_content)
    }
    /// ensure the existance module dir
    fn ensure_module_dir(&mut self) -> Result<()> {
        fs::create_dir_all(&self.modules_path)
    }
    /// copy or replace module
    pub fn copy_module(&mut self, module_path_from: &Path) -> Result<u64> {
        self.ensure_module_dir()?;
        let module_path_to = self
            .modules_path
            .join(module_path_from.file_name().unwrap_or(OsStr::new("unknown_module")));
        println!(
            "copying module from: {} to: {}",
            module_path_to.display(),
            module_path_from.display()
        ); // replace with logging
        fs::copy(module_path_from, module_path_to)
    }
    /// return prefix
    pub fn prefix(&mut self) -> &Path {
        &self.prefix
    }
}

#[cfg(target_os = "macos")]
fn target_cands() -> Option<Vec<PathBuf>> {
    match std::env::var("DYLD_FALLBACK_LIBRARY_PATH") {
        Ok(cands) => Some(cands.split(':').map(PathBuf::from).collect()),
        Err(_) => None,
    }
}
#[cfg(target_os = "linux")]
fn target_dir_cands() -> Option<Vec<PathBuf>> {
    match std::env::var("LD_LIBRARY_PATH") {
        Ok(cands) => Some(cands.split(':').map(PathBuf::from).collect()),
        Err(_) => None,
    }
}

/// search path and return the path to the target
pub fn target_path(target_name: &str) -> std::io::Result<PathBuf> {
    if let Some(cands) = target_cands() {
        for dir in cands {
            if let Ok(iter) = read_dir(dir) {
                for entry in iter {
                    if let Ok(entry) = entry {
                        if entry.file_name() == target_name {
                            return Ok(entry.path());
                        }
                    }
                }
            }
        }
    }
    Err(std::io::ErrorKind::NotFound.into())
}
