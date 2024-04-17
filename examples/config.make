ngx_addon_name=ngx_rust_examples
ngx_cargo_profile=ngx-module
ngx_cargo_manifest=$(realpath $ngx_addon_dir/Cargo.toml)
ngx_cargo_features=
ngx_rust_examples="async awssig curl upstream"

case "$NGX_PLATFORM" in
    Linux:*)
		ngx_cargo_features="$ngx_cargo_features linux"
		ngx_rust_examples="$ngx_rust_examples httporigdst"
        ;;
esac

for ngx_rust_example in $ngx_rust_examples
do

cat << END                                            >> $NGX_MAKEFILE

# Always call cargo instead of tracking the source modifications
.PHONY: $NGX_OBJS/$ngx_addon_name/$ngx_cargo_profile/examples/lib$ngx_rust_example.a

$NGX_OBJS/$ngx_addon_name/$ngx_cargo_profile/examples/lib$ngx_rust_example.a:
	cd $NGX_OBJS && \\
	NGX_OBJS="\$\$PWD" cargo rustc \\
		--crate-type staticlib \\
		--example "$ngx_rust_example" \\
		--no-default-features \\
		--features "$ngx_cargo_features" \\
		--profile $ngx_cargo_profile \\
		--target-dir $ngx_addon_name \\
		--manifest-path $ngx_cargo_manifest

END

done
