NGINX_VER = 1.11.13
UNAME_S := $(shell uname -s)
NGX_MODULES = --with-compat  --with-threads --with-http_addition_module \
     --with-http_auth_request_module   --with-http_gunzip_module --with-http_gzip_static_module  \
     --with-http_random_index_module --with-http_realip_module --with-http_secure_link_module \
     --with-http_slice_module  --with-http_stub_status_module --with-http_sub_module \
     --with-stream --with-stream_realip_module --with-stream_ssl_preread_module

ifeq ($(UNAME_S),Linux)
    NGINX_SRC += nginx-linux
    NGX_OPT= $(NGX_MODULES) \
       --with-file-aio --with-http_ssl_module --with-stream_ssl_module  \
       --with-cc-opt='-g -fstack-protector-strong -Wformat -Werror=format-security -Wp,-D_FORTIFY_SOURCE=2 -fPIC' \
       --with-ld-opt='-Wl,-Bsymbolic-functions -Wl,-z,relro -Wl,-z,now -Wl,--as-needed -pie'
endif
ifeq ($(UNAME_S),Darwin)
    NGINX_SRC += nginx-darwin
    NGX_OPT= $(NGX_MODULES)
endif
MODULE_NAME=ngx_rust
MODULE_LIB=${MODULE_SRC}/nginx-${NGINX_VER}/objs/${MODULE_NAME}.so
NGX_DEBUG="--with-debug"
RUST_COMPILER_TAG = 1.20.0
RUST_TOOL = nginmesh/ngx-rust-tool:${RUST_COMPILER_TAG}
export ROOT_DIR=${PWD}

nginx-build:
	cd nginx/${NGINX_SRC}; \
	./configure --prefix=${PWD}/nginx/install $(NGX_OPT); \
	make; \
	make install


setup-nginx:
	mkdir -p nginx

nginx-source:	setup-nginx
	rm -rf nginx/${NGINX_SRC}
	wget http://nginx.org/download/nginx-${NGINX_VER}.tar.gz
	tar zxf nginx-${NGINX_VER}.tar.gz
	mv nginx-${NGINX_VER} ${NGINX_SRC}
	mv ${NGINX_SRC} nginx
	rm nginx-${NGINX_VER}.tar.gz*

nginx-configure:
	cd nginx/${NGINX_SRC}; \
    ./configure --add-dynamic-module=../../module $(NGX_OPT)


nginx-setup:	nginx-source nginx-configure

nginx-test:	nginx-source nginx-build


nginx-module:
	cd nginx/${NGINX_SRC}; \
	make modules;

# need to run inside container
linux-shell:
	docker run -it -v ${ROOT_DIR}:/src ${RUST_TOOL}  /bin/bash

cargo:
	docker run -it -v ${ROOT_DIR}:/src -w /src/ ${RUST_TOOL} cargo build

linux-setup:
	docker run -it -v ${ROOT_DIR}:/src -w /src/ ${RUST_TOOL} make nginx-setup


linux-module:
	docker run -it -v ${ROOT_DIR}:/src -w /src/ ${RUST_TOOL} make nginx-module



clean:
	cargo clean
	rm -f src/bindings.rs


super_clean: clean
	rm -rf nginx
