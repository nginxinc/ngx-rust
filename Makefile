MODULE_NAME=ngx_http_rust_module
MODULE_PROJ_NAME=ngx-rust
NGX_DEBUG="--with-debug"
include nginx.mk


clean:
	cargo clean
	rm -f src/bindings.rs


super_clean: clean
	rm -rf nginx
