extern crate ngx_rust;
extern crate futures;
extern crate hyper;
extern crate tokio_core;


#[cfg(test)]
mod tests {

    use std::env;
    use ngx_rust::nginx::Nginx;
    use futures::Future;
    use hyper::Client;
    use hyper::StatusCode;
    use tokio_core::reactor::Core;

    const TEST_NGINX_CONFIG: &str = "tests/nginx.conf";


    #[test]
    fn test() {

        let mut nginx = Nginx::default();

        let path = env::current_dir().unwrap();
        let test_config_path = format!("{}/{}",path.display(),TEST_NGINX_CONFIG);
        (nginx.replace_config(&test_config_path)).expect("copy done");
        let output =  nginx.restart().expect("fail to start");
        assert!(output.status.success());

        // make request to 30000

        let mut core = Core::new().unwrap();

        let client = Client::new(&core.handle());

        let uri = "http://localhost:30000".parse().unwrap();
        let work = client.get(uri).map(|res| {
            let status = res.status();
            println!("Response: {}", status);
            assert_eq!(status,StatusCode::Ok);

        });

        core.run(work).unwrap();
    }


}