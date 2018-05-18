NGINX_VER = 1.13.11

clean:
	cargo clean
	make -C ngx-sys clean
