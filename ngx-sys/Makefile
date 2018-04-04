NGINX_VER = 1.13.11

UNAME_S := $(shell uname -s)
NGX_MODULES = --with-compat  --with-threads --with-http_addition_module \
     --with-http_auth_request_module   --with-http_gunzip_module --with-http_gzip_static_module  \
     --with-http_random_index_module --with-http_realip_module --with-http_secure_link_module \
     --with-http_slice_module  --with-http_stub_status_module --with-http_sub_module \
     --with-stream --with-stream_realip_module --with-stream_ssl_preread_module

ifeq ($(UNAME_S),Linux)
    NGINX_SRC = nginx-linux
    NGX_OPT= $(NGX_MODULES) \
       --with-file-aio --with-http_ssl_module --with-stream_ssl_module  \
       --with-cc-opt='-g -fstack-protector-strong -Wformat -Werror=format-security -Wp,-D_FORTIFY_SOURCE=2 -fPIC' \
       --with-ld-opt='-Wl,-Bsymbolic-functions -Wl,-z,relro -Wl,-z,now -Wl,--as-needed -pie'
endif
ifeq ($(UNAME_S),Darwin)
    NGINX_SRC = nginx-darwin
    NGX_OPT= $(NGX_MODULES)
endif

NGX_DEBUG="--with-debug"

configure:
	echo "using opt: $(NGX_OPT)"
	cd nginx; bash auto/configure $(NGX_OPT)

