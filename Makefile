NGINX_VER = 1.11.13
MODULE_NAME=ngx_rust
MODULE_LIB=${MODULE_SRC}/nginx-${NGINX_VER}/objs/${MODULE_NAME}.so
NGX_DEBUG="--with-debug"
DARWIN_NGINX=nginx-darwin
LINUX_NGINX=nginx-linux

darwin-source:
	wget http://nginx.org/download/nginx-${NGINX_VER}.tar.gz
	tar zxf nginx-${NGINX_VER}.tar.gz
	mv nginx-${NGINX_VER} ${DARWIN_NGINX}
	mv ${DARWIN_NGINX} nginx
	rm nginx-${NGINX_VER}.tar.gz*

darwin-configure:
	cd nginx/${DARWIN_NGINX}; \
    ./configure --add-dynamic-module=../../module

darwin-setup:   darwin-source darwin-configure



clean:
	cargo clean
	rm -f src/bindings.rs


super_clean: clean
	rm -rf nginx


report:
	cargo build --bin report_client

test:	
	cargo test -- --nocapture
