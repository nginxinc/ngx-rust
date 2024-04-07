#[cfg(test)]
mod tests {
    use ngx::test_util::Nginx;
    use std::env::current_dir;

    const TEST_NGINX_CONFIG: &str = "tests/nginx.conf";

    #[test]
    fn test() {
        let mut nginx = Nginx::default();

        let current_dir = current_dir().expect("Unable to get current directory");
        let test_config_path = current_dir.join(TEST_NGINX_CONFIG);

        assert!(
            test_config_path.is_file(),
            "Config file not found: {}\nCurrent directory: {}",
            test_config_path.to_string_lossy(),
            current_dir.to_string_lossy()
        );

        nginx
            .copy_config(&test_config_path)
            .expect(format!("Unable to load config file: {}", test_config_path.display()).as_str());
        let output = nginx.restart().expect("Unable to restart NGINX");
        assert!(output.status.success());

        let output = nginx.stop().expect("Unable to stop NGINX");
        assert!(output.status.success());
    }
}
