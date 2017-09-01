extern crate ngx_rust;

#[cfg(test)]
mod tests {

    use std::env;
    use ngx_rust::nginx::Nginx;

    const NGINX_BIN: &str = "nginx/install/sbin/nginx";

    #[test]
    fn test() {

        let mut nginx = Nginx::default();
        let output =  nginx.restart().expect("fail to start");

        assert!(output.status.success());

    }


}