
cat << END                                                    >> $NGX_MAKEFILE

cargo:
	cargo build --manifest-path $ngx_addon_dir/../Cargo.toml --lib

END
