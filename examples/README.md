# Examples

## NGINX Destination IP recovery module for HTTP

This dynamic module recovers original IP address and port number of the destination packet. It is useful, for example, with container sidecars where all outgoing traffic is redirected to a separate container with iptables before reaching the target.

This module can only be built with the "linux" feature enabled, and will only successfully build on a Linux OS.

### Dependencies

This modules uses the Rust crate libc and Linux **getsockopt** socket API.

### Example Configuration
#### HTTP

```nginx configuration
load_module "modules/libhttporigdst.so";

http {
    server {
        # use iptables to capture all outgoing traffic and REDIRECT
        # to listening port 15501
        listen 15501;

        # binding variables provided by module will lazily activate it
        # and store a context
        # variables can be used in config
        location / {
            # Return if no backend is available or proxy_pass
            # return 200 "recv'd: $server_addr:$server_port\n\nproxy_pass http://$server_orig_addr:$server_orig_port\n";
            proxy_pass http://$server_orig_addr:$server_orig_port;
        }
    }
}
```

### Embedded Variables

The following embedded variables are provided:

* **server_orig_addr**
  * Original IP address
* **server_orig_port**
  * Original port

### Usage

1. Clone the git repository.
  ```
  https://github.com/nginxinc/ngx-rust
  ```

2. Compile the module from the cloned repo.
  ```
  cd ${CLONED_DIRECTORY}/ngx-rust
  cargo build --package=examples --example=httporigdst --features=linux
  ```

3. Copy the shared object to the modules directory, /etc/nginx/modules.
  ```
  cp ./target/debug/examples/libhttporigdst.so /etc/nginx/modules
  ```

4. Add the `load_module` directive to your configuration.
  ```
  load_module "modules/libhttporigdst.so";
  ```

5. Reload NGINX.
  ```
  nginx -t && nginx -s reload
  ```

6. Redirect traffic outbound.
  ```
  iptables -t nat -N NGINX_REDIRECT && \
  iptables -t nat -A NGINX_REDIRECT -p tcp -j REDIRECT --to-port 15501 --dport 15001 && \
  iptables -t nat -N NGINX_OUTPUT && \
  iptables -t nat -A OUTPUT -p tcp -j NGINX_OUTPUT && \
  iptables -t nat -A NGINX_OUTPUT -j NGINX_REDIRECT
  ```

7. Redirect traffic inbound.
  ```
  iptables -t nat -N NGINX_IN_REDIRECT && \
  iptables -t nat -A NGINX_IN_REDIRECT -p tcp -j REDIRECT --to-port 15501 --dport 15001 && \
  iptables -t nat -N NGINX_INBOUND && \
  iptables -t nat -A PREROUTING -p tcp -j NGINX_INBOUND && \
  iptables -t nat -A NGINX_INBOUND -p tcp -j NGINX_IN_REDIRECT
  ```

8. Test with `curl` (this step assumes you've uncommented the return directive).
  ```
  curl --output - ${LISTEN_IP}:15001
  recv'd: ${LISTEN_IP}:15501

  proxy_pass http://${LISTEN_IP}:15001
  ```
### Caveats

This module only supports IPv4.
