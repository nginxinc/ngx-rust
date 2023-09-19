extern crate bindgen;
extern crate duct;

use duct::cmd;
use flate2::read::GzDecoder;
use std::error::Error as StdError;
use std::ffi::OsString;
use std::fs::File;
use std::io::Error as IoError;
use std::io::ErrorKind::NotFound;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::{env, thread};
use tar::Archive;
use which::which;

/// The default version of zlib to use if the `ZLIB_VERSION` environment variable is not present
const ZLIB_DEFAULT_VERSION: &str = "1.3";
const ZLIB_GPG_SERVER_AND_KEY_ID: (&str, &str) = ("keyserver.ubuntu.com", "783FCD8E58BCAFBA");
const ZLIB_DOWNLOAD_URL_PREFIX: &str = "https://www.zlib.net";
/// The default version of pcre2 to use if the `PCRE2_VERSION` environment variable is not present
const PCRE2_DEFAULT_VERSION: &str = "10.42";
const PCRE2_GPG_SERVER_AND_KEY_ID: (&str, &str) = ("keyserver.ubuntu.com", "9766E084FB0F43D8");
const PCRE2_DOWNLOAD_URL_PREFIX: &str = "https://github.com/PCRE2Project/pcre2/releases/download";
/// The default version of openssl to use if the `OPENSSL_VERSION` environment variable is not present
const OPENSSL_DEFAULT_VERSION: &str = "3.0.7";
const OPENSSL_GPG_SERVER_AND_KEY_IDS: (&str, &str) = (
    "keys.openpgp.org",
    "\
A21FAB74B0088AA361152586B8EF1A6BA9DA2D5C \
8657ABB260F056B1E5190839D9C4D26D0E604491 \
B7C1C14360F353A36862E4D5231C84CDDCC69C45 \
95A9908DDFA16830BE9FB9003D30A3A9FF1360DC \
7953AC1FBC3DC8B3B292393ED5E9E43F7DF9EE8C",
);
const OPENSSL_DOWNLOAD_URL_PREFIX: &str = "https://www.openssl.org/source/";
/// The default version of NGINX to use if the `NGX_VERSION` environment variable is not present
const NGX_DEFAULT_VERSION: &str = "1.23.3";
const NGX_GPG_SERVER_AND_KEY_ID: (&str, &str) = ("keyserver.ubuntu.com", "A0EA981B66B0D967");
const NGX_DOWNLOAD_URL_PREFIX: &str = "https://nginx.org/download";
/// If you are adding another dependency, you will need to add the server/public key tuple below.
const ALL_DEVPS_SERVERS_AND_PUBLIC_KEY_IDS: [(&str, &str); 3] = [
    ZLIB_GPG_SERVER_AND_KEY_ID,
    PCRE2_GPG_SERVER_AND_KEY_ID,
    OPENSSL_GPG_SERVER_AND_KEY_IDS,
];
/// List of configure switches specifying the modules to build NGINX with
const NGX_BASE_MODULES: [&str; 19] = [
    "--with-compat",
    "--with-http_addition_module",
    "--with-http_auth_request_module",
    "--with-http_flv_module",
    "--with-http_gunzip_module",
    "--with-http_gzip_static_module",
    "--with-http_random_index_module",
    "--with-http_realip_module",
    "--with-http_secure_link_module",
    "--with-http_slice_module",
    "--with-http_ssl_module",
    "--with-http_stub_status_module",
    "--with-http_sub_module",
    "--with-http_v2_module",
    "--with-stream_realip_module",
    "--with-stream_ssl_module",
    "--with-stream_ssl_preread_module",
    "--with-stream",
    "--with-threads",
];
/// Additional configuration flags to use when building on Linux.
const NGX_LINUX_ADDITIONAL_OPTS: [&str; 1] = [
    "--with-file-aio",
    // TODO: do we really need, ngx-rust is not intended to compile nginx, we need only header files
    // "--with-cc-opt='-g,-fstack-protector-strong,-Wformat,-Werror=format-security,-Wp,-D_FORTIFY_SOURCE=2,-fPIC'",
    // "--with-ld-opt='-Wl,-Bsymbolic-functions -Wl,-z,relro -Wl,-z,now -Wl,--as-needed,-pie'",
];
/// List of env vars that trigger builds.rs to re-run (rerun-if-env-changed)
const ENV_VARS_TRIGGERING_RECOMPILE: [&str; 11] = [
    "DEBUG",
    "OUT_DIR",
    "ZLIB_VERSION",
    "PCRE2_VERSION",
    "OPENSSL_VERSION",
    "NGX_VERSION",
    "NGX_SRC_DIR",
    "NGX_CONFIGURE_ARGS",
    "CARGO_CFG_TARGET_OS",
    "CARGO_MANIFEST_DIR",
    "CARGO_TARGET_TMPDIR",
];

/// Function invoked when `cargo build` is executed.
/// This function will download NGINX and all supporting dependencies, verify their integrity,
/// extract them, execute autoconf `configure` for NGINX, compile NGINX and finally install
/// NGINX in a subdirectory with the project or use provided NGNINX source path
fn main() -> Result<(), Box<dyn StdError>> {
    let src_dir = nginx_src_dir()?;

    configure_nginx(&src_dir)?;

    println!("cargo:rerun-if-changed={}/objs/Makefile", src_dir.display());

    // Hint cargo to rebuild if any of the these environment variables values change
    // because they will trigger a recompilation of NGINX with different parameters
    for var in ENV_VARS_TRIGGERING_RECOMPILE {
        println!("cargo:rerun-if-env-changed={var}");
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    // Read autoconf generated makefile for NGINX and generate Rust bindings based on its includes
    generate_binding(src_dir);
    Ok(())
}

/// Generates Rust bindings for NGINX
fn generate_binding(nginx_source_dir: PathBuf) {
    let autoconf_makefile_path = nginx_source_dir.join("objs").join("Makefile");
    let clang_args: Vec<String> = parse_includes_from_makefile(&autoconf_makefile_path)
        .into_iter()
        .map(|path| format!("-I{}", path.to_string_lossy()))
        .collect();

    println!("bindgen clang_args: {:?}", clang_args);
    let bindings = bindgen::Builder::default()
        // Bindings will not compile on Linux without block listing this item
        // It is worth investigating why this is
        .blocklist_item("IPPORT_RESERVED")
        // The input header we would like to generate bindings for.
        .header("wrapper.h")
        .allowlist_type("ngx_.*")
        .allowlist_function("ngx_.*")
        .allowlist_var("ngx_.*|NGX_.*|nginx_.*")
        .clang_args(clang_args)
        .layout_tests(false)
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_dir_env = env::var("OUT_DIR").expect("The required environment variable OUT_DIR was not set");
    let out_path = PathBuf::from(out_dir_env);
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

/*
###########################################################################
# NGINX Build Functions - Everything below here is for building NGINX     #
###########################################################################

In order to build Rust bindings for NGINX using the bindgen crate, we need
to do the following:

 1. Download NGINX source code and all dependencies (zlib, pcre2, openssl)
 2. Verify the integrity of the downloaded files using GPG signatures
 3. Extract the downloaded files
 4. Run autoconf `configure` for NGINX
 5. Compile NGINX
 6. Install NGINX in a subdirectory of the project
 7. Read the autoconf generated makefile for NGINX and configure bindgen
    to generate Rust bindings based on the includes in the makefile.

Additionally, we want to provide the following features as part of the
build process:
 * Allow the user to specify the version of NGINX to build
 * Allow the user to specify the version of each dependency to build
 * Only reconfigure and recompile NGINX if any of the above versions
   change or the configuration flags change (like enabling or disabling
   the debug mode)
 * Not rely on the user having NGINX dependencies installed on their
   system (zlib, pcre2, openssl)
 * Keep source code and binaries confined to a subdirectory of the
   project to avoid having to track files outside of the project
 * If GPG is not installed, the build will still continue. However, the
   integrity of the downloaded files will not be verified.
*/

fn zlib_archive_url() -> String {
    let version = env::var("ZLIB_VERSION").unwrap_or_else(|_| ZLIB_DEFAULT_VERSION.to_string());
    format!("{ZLIB_DOWNLOAD_URL_PREFIX}/zlib-{version}.tar.gz")
}

fn pcre2_archive_url() -> String {
    let version = env::var("PCRE2_VERSION").unwrap_or_else(|_| PCRE2_DEFAULT_VERSION.to_string());
    format!("{PCRE2_DOWNLOAD_URL_PREFIX}/pcre2-{version}/pcre2-{version}.tar.gz")
}

fn openssl_archive_url() -> String {
    let version = env::var("OPENSSL_VERSION").unwrap_or_else(|_| OPENSSL_DEFAULT_VERSION.to_string());
    format!("{OPENSSL_DOWNLOAD_URL_PREFIX}/openssl-{version}.tar.gz")
}

fn nginx_archive_url() -> String {
    let version = env::var("NGX_VERSION").unwrap_or_else(|_| NGX_DEFAULT_VERSION.to_string());
    format!("{NGX_DOWNLOAD_URL_PREFIX}/nginx-{version}.tar.gz")
}

/// Returns a list of tuples containing the URL to a tarball archive and the GPG signature used
/// to validate the integrity of the tarball.
fn all_dep_archives() -> Vec<(String, String)> {
    vec![
        (zlib_archive_url(), format!("{}.asc", zlib_archive_url())),
        (pcre2_archive_url(), format!("{}.sig", pcre2_archive_url())),
        (openssl_archive_url(), format!("{}.asc", openssl_archive_url())),
    ]
}

fn gpg_path() -> Option<PathBuf> {
    which::which("gpg").ok()
}

/// Returns the base path to extract tarball contents into
fn source_output_dir(cache_dir: &Path) -> PathBuf {
    env::var("CARGO_TARGET_TMPDIR").map(PathBuf::from).unwrap_or_else(|_| {
        cache_dir
            .join("src")
            .join(format!("{}-{}", env::consts::OS, env::consts::ARCH))
    })
}
/// Returns path to NGINX source code
/// If env NGX_SRC_DIR var is set use it, otherwise download NGINX OSS from the Internet
fn nginx_src_dir() -> Result<PathBuf, Box<dyn StdError>> {
    let nginx_src_dir = match env::var("NGX_SRC_DIR") {
        Err(_) => {
            // Create .cache directory
            let cache_dir = make_cache_dir()?;
            // Import GPG keys used to verify dependency tarballs
            import_gpg_keys(&cache_dir, &[NGX_GPG_SERVER_AND_KEY_ID; 1])?;
            let extract_output_base_dir = source_output_dir(&cache_dir);
            if !extract_output_base_dir.exists() {
                std::fs::create_dir_all(&extract_output_base_dir)?;
            }
            // download nginx from the Internet and extract it to the cache folder
            let archive_url = nginx_archive_url();
            let signature_url = format!("{}.asc", archive_url);
            let archive_path = get_archive(&cache_dir, &archive_url, &signature_url)?;
            let (_, output_dir) = extract_archive(&archive_path, &extract_output_base_dir)?;
            output_dir
        }
        Ok(v) => PathBuf::from(v),
    };
    Ok(nginx_src_dir)
}

/// Returns Path to the dependency or panics is not found
fn find_dependency_path<'a>(sources: &'a [(String, PathBuf)], name: &str) -> &'a Path {
    sources
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, p)| p.as_path())
        .unwrap_or_else(|| panic!("Unable to find dependency [{name}] path"))
}

#[allow(clippy::ptr_arg)]
/// Returns the path to install NGINX to
fn nginx_install_dir(base_dir: &PathBuf) -> PathBuf {
    let nginx_version = env::var("NGX_VERSION").unwrap_or_else(|_| NGX_DEFAULT_VERSION.to_string());
    let platform = format!("{}-{}", env::consts::OS, env::consts::ARCH);
    base_dir.join("nginx").join(nginx_version).join(platform)
}

/// Imports GPG keys into the `.cache/.gnupu` directory in order to
/// validate the integrity of the downloaded tarballs.
fn import_gpg_keys(cache_dir: &Path, keys: &[(&str, &str)]) -> Result<(), Box<dyn StdError>> {
    if let Some(gpg) = gpg_path() {
        // We do not want to mess with the default gpg data for the running user,
        // so we store all gpg data with our cache directory.
        let gnupghome = cache_dir.join(".gnupg");
        if !gnupghome.exists() {
            std::fs::create_dir_all(&gnupghome)?;
        }

        let keys_to_import = keys.iter().filter(|(_, key_id)| {
            let key_id_record_file = gnupghome.join(format!("{key_id}.key"));
            !key_id_record_file.exists()
        });

        for (server, key_ids) in keys_to_import {
            for key_id in key_ids.split_whitespace() {
                let output = cmd!(
                    &gpg,
                    "--homedir",
                    &gnupghome,
                    "--keyserver",
                    server,
                    "--recv-keys",
                    key_id
                )
                .stderr_to_stdout()
                .stderr_capture()
                .run()?;
                if !output.status.success() {
                    return Err(format!(
                        "Failed to import GPG key {} from server {}: {}",
                        key_id,
                        server,
                        String::from_utf8_lossy(&output.stdout)
                    )
                    .into());
                }
                println!("Imported GPG key: {key_id}");
                let key_id_record_file = gnupghome.join(format!("{key_ids}.key"));
                File::create(key_id_record_file).expect("Unable to create key id record file");
            }
        }
    }
    Ok(())
}

fn make_cache_dir() -> Result<PathBuf, Box<dyn StdError>> {
    let base_dir = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::current_dir().expect("Failed to get current directory"));
    // Choose the parent directory of the manifest directory (nginx-sys) as the cache directory
    // Fail if we do not have a parent directory
    let cache_dir = base_dir
        .parent()
        .expect("Failed to find parent directory of manifest directory")
        .join(".cache");
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)?;
    }
    Ok(cache_dir)
}

/// Downloads a tarball from the specified URL into the `.cache` directory.
fn download(cache_dir: &Path, url: &str) -> Result<PathBuf, Box<dyn StdError>> {
    fn proceed_with_download(file_path: &Path) -> bool {
        // File does not exist or is zero bytes
        !file_path.exists() || file_path.metadata().map_or(false, |m| m.len() < 1)
    }
    let filename = url.split('/').last().unwrap();
    let file_path = cache_dir.join(filename);
    if proceed_with_download(&file_path) {
        let mut reader = ureq::get(url).call()?.into_reader();
        let mut file = std::fs::File::create(&file_path)?;
        std::io::copy(&mut reader, &mut file)?;
    }
    Ok(file_path)
}

/// Validates that a file is a valid GPG signature file.
fn verify_signature_file(cache_dir: &Path, signature_path: &Path) -> Result<(), Box<dyn StdError>> {
    if let Some(gpg) = gpg_path() {
        let gnupghome = cache_dir.join(".gnupg");
        let output = cmd!(gpg, "--homedir", &gnupghome, "--list-packets", signature_path)
            .stderr_to_stdout()
            .stdout_capture()
            .run()?;
        if !output.status.success() {
            eprintln!("{}", String::from_utf8_lossy(&output.stdout));
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "GPG signature file verification failed for signature: {}",
                    signature_path.display()
                ),
            )));
        }
    } else {
        println!("GPG not found, skipping signature file verification");
    }
    Ok(())
}

/// Validates the integrity of a tarball file against the cryptographic signature associated with
/// the file.
fn verify_archive_signature(
    cache_dir: &Path,
    archive_path: &Path,
    signature_path: &Path,
) -> Result<(), Box<dyn StdError>> {
    if let Some(gpg) = gpg_path() {
        let gnupghome = cache_dir.join(".gnupg");
        let output = cmd!(gpg, "--homedir", &gnupghome, "--verify", signature_path, archive_path)
            .stderr_to_stdout()
            .stdout_capture()
            .run()?;
        if !output.status.success() {
            eprintln!("{}", String::from_utf8_lossy(&output.stdout));
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "GPG signature verification failed of archive failed [{}]",
                    archive_path.display()
                ),
            )));
        }
    } else {
        println!("GPG not found, skipping signature verification");
    }
    Ok(())
}

/// Get a given tarball and signature file from a remote URL and copy it to the `.cache` directory.
fn get_archive(cache_dir: &Path, archive_url: &str, signature_url: &str) -> Result<PathBuf, Box<dyn StdError>> {
    let signature_path = download(cache_dir, signature_url)?;
    if let Err(e) = verify_signature_file(cache_dir, &signature_path) {
        std::fs::remove_file(&signature_path)?;
        return Err(e);
    }
    let archive_path = download(cache_dir, archive_url)?;
    match verify_archive_signature(cache_dir, &archive_path, &signature_path) {
        Ok(_) => Ok(archive_path),
        Err(e) => {
            std::fs::remove_file(&archive_path)?;
            Err(e)
        }
    }
}

/// Extract a tarball into a subdirectory based on the tarball's name under the source base
/// directory.
fn extract_archive(
    archive_path: &Path,
    extract_output_base_dir: &Path,
) -> Result<(String, PathBuf), Box<dyn StdError>> {
    if !extract_output_base_dir.exists() {
        std::fs::create_dir_all(extract_output_base_dir)?;
    }
    let archive_file =
        File::open(archive_path).unwrap_or_else(|_| panic!("Unable to open archive file: {}", archive_path.display()));
    let stem = archive_path
        .file_name()
        .and_then(|s| s.to_str())
        .and_then(|s| s.rsplitn(3, '.').last())
        .expect("Unable to determine archive file name stem");
    let dependency_name = stem
        .split_once('-')
        .map(|(s, _)| s.to_owned())
        .unwrap_or_else(|| panic!("Unable to determine dependency name based on stem: {stem}"));

    let extract_output_dir = extract_output_base_dir.to_owned();
    let archive_output_dir = extract_output_dir.join(stem);
    if !archive_output_dir.exists() {
        Archive::new(GzDecoder::new(archive_file))
            .entries()?
            .filter_map(|e| e.ok())
            .for_each(|mut entry| {
                let path = entry.path().unwrap();
                let stripped_path = path.components().skip(1).collect::<PathBuf>();
                entry.unpack(&archive_output_dir.join(stripped_path)).unwrap();
            });
    } else {
        println!(
            "Archive [{}] already extracted to directory: {}",
            stem,
            archive_output_dir.display()
        );
    }

    Ok((dependency_name, archive_output_dir))
}

/// Extract dependencies of the tarballs into subdirectories within the source base directory.
fn extract_all_dep_archives(cache_dir: &Path) -> Result<Vec<(String, PathBuf)>, Box<dyn StdError>> {
    let archives = all_dep_archives();
    let mut sources = Vec::new();
    let extract_output_base_dir = source_output_dir(cache_dir);
    if !extract_output_base_dir.exists() {
        std::fs::create_dir_all(&extract_output_base_dir)?;
    }

    import_gpg_keys(cache_dir, &ALL_DEVPS_SERVERS_AND_PUBLIC_KEY_IDS)?;
    for (archive_url, signature_url) in archives {
        let archive_path = get_archive(cache_dir, &archive_url, &signature_url)?;
        let (name, output_dir) = extract_archive(&archive_path, &extract_output_base_dir)?;
        sources.push((name, output_dir));
    }

    Ok(sources)
}

/// Executes `configure` script and `make build` for NGINX if wasn't ran previously
/// Reads env NGX_CONFIGURE_ARGS var or uses predefnied options and dependecies
fn configure_nginx(nginx_src_dir: &PathBuf) -> Result<(), Box<dyn StdError>> {
    let flags = match env::var("NGX_CONFIGURE_ARGS") {
        Ok(v) => v,
        Err(_) => {
            let cache_dir = make_cache_dir()?;
            let nginx_install_dir = nginx_install_dir(&cache_dir);
            let sources = extract_all_dep_archives(&cache_dir)?;
            let zlib_src_dir = find_dependency_path(&sources, "zlib");
            let openssl_src_dir = find_dependency_path(&sources, "openssl");
            let pcre2_src_dir = find_dependency_path(&sources, "pcre2");
            let args = nginx_configure_flags(&nginx_install_dir, zlib_src_dir, openssl_src_dir, pcre2_src_dir);
            args.join(" ")
        }
    };

    let autoconf_makefile_exists = autoconf_exists(nginx_src_dir);
    // NGINX binary file is in objs folder after `make build`
    let nginx_binary_exists = nginx_src_dir.join("objs").join("nginx").exists();

    println!("NGINX autoconf makefile already created: {autoconf_makefile_exists}");
    println!("NGINX source folder: {:?}", nginx_src_dir);
    println!("NGINX already built: {nginx_binary_exists}");
    println!("NGINX configure flags: {flags}");

    // TODO: a problem: NGX_SRC_DIR was provided and user already ran configure/build
    // but did not provide NGX_CONFIGURE_ARGS and the flags used are different with our default.
    // what do to?
    if !autoconf_makefile_exists {
        run_configure_nginx(nginx_src_dir, &flags)?;
    }

    // TODO: a problem why we need to ran build - openssl needs to be configured if it is used,
    // potentially openssl src path can be whatever NGX_CONFIGURE_ARGS or defaults to the OS
    // but when `configure` is ran for nginx it doesn't configure openssl. openssl is configured only when
    // running make build. if make build wasn't run then openssl is not configured and rust bingen can
    // 'Unable to generate bindings fails like: ClangDiagnostic("<ommited>.cache/src/macos-x86_64/nginx-1.23.3/src/event/ngx_event_openssl.h:17:10: fatal error: 'openssl/ssl.h' file not found\n")',
    // this is because objs/Makefile contains
    // `-I/"<ommited>.cache/src/macos-x86_64/openssl-3.0.7/.openssl/include`, where `.openssl/include` is result of configuring openssl
    // so let's check and ran make build at least once
    if !nginx_binary_exists {
        // at least once let's run make
        // when NGX_CONFIGURE_ARGS provided don't install it as we hmay not trust this
        if env::var("NGX_CONFIGURE_ARGS").is_ok() {
            make(nginx_src_dir, "build")?;
        } else {
            make(nginx_src_dir, "install")?;
        }
    }
    Ok(())
}

/// check if nginx was autoconfigured already by checking Makefile
fn autoconf_exists(nginx_src_dir: &Path) -> bool {
    nginx_src_dir.join("Makefile").exists()
}

/// Generate the flags to use with autoconf `configure` for NGINX based on the downloaded
/// dependencies' paths. Note: the paths differ based on cargo targets because they may be
/// configured differently for different os/platform targets.
fn nginx_configure_flags(
    nginx_install_dir: &Path,
    zlib_src_dir: &Path,
    openssl_src_dir: &Path,
    pcre2_src_dir: &Path,
) -> Vec<String> {
    fn format_source_path(flag: &str, path: &Path) -> String {
        format!(
            "{}={}",
            flag,
            path.as_os_str().to_str().expect("Unable to read source path as string")
        )
    }
    let modules = || -> Vec<String> {
        let mut modules = Vec::new();
        for module in NGX_BASE_MODULES {
            modules.push(module.to_string());
        }
        modules.push(format_source_path("--with-zlib", zlib_src_dir));
        modules.push(format_source_path("--with-pcre", pcre2_src_dir));
        modules.push(format_source_path("--with-openssl", openssl_src_dir));
        modules
    };
    let mut nginx_opts = vec![format_source_path("--prefix", nginx_install_dir)];
    if env::var("NGX_DEBUG").map_or(false, |s| s == "true") {
        println!("Enabling --with-debug");
        nginx_opts.push("--with-debug".to_string());
    }
    if env::var("CARGO_CFG_TARGET_OS").map_or(env::consts::OS == "linux", |s| s == "linux") {
        for flag in NGX_LINUX_ADDITIONAL_OPTS {
            nginx_opts.push(flag.to_string());
        }
    }
    for flag in modules() {
        nginx_opts.push(flag);
    }
    nginx_opts
}

/// Run external process invoking autoconf `configure` for NGINX.
fn run_configure_nginx(nginx_src_dir: &Path, flags: &str) -> std::io::Result<Output> {
    let mut configure_executable = nginx_src_dir.join("auto").join("configure");
    if !configure_executable.exists() {
        println!("checking NGINX configure on the top level...");
        // in some cases configure is located at the top level (gzip sources download from the nginx.org)
        configure_executable = nginx_src_dir.join("configure");

        if !configure_executable.exists() {
            panic!(
                "Unable to find NGINX configure script at: {}",
                configure_executable.to_string_lossy()
            );
        }
    }
    println!(
        "NGINX configure script was found: {}",
        configure_executable.to_string_lossy()
    );
    // FIXME: it might cause an issue incorectly splitting argumens
    // if it contains `--with-cc-opt='-g -fstack-protector-strong'`
    // then it would split in ["--with-cc-opt='-g", "-fstack-protector-strong"] which is not correct,
    // there is no such argument `-fstack-protector-strong` for nginx's configure script
    // instead probably best is to use CFLAGS env or other way..
    let args = flags
        .split_ascii_whitespace()
        .map(OsString::from)
        .collect::<Vec<OsString>>();

    duct::cmd(&configure_executable, args)
        .dir(nginx_src_dir)
        .stderr_to_stdout()
        .run()
}

/// Run `make` within the NGINX source directory as an external process.
fn make(nginx_src_dir: &Path, arg: &str) -> std::io::Result<Output> {
    // Give preference to the binary with the name of gmake if it exists because this is typically
    // the GNU 4+ on MacOS (if it is installed via homebrew).
    let make_bin_path = match (which("gmake"), which("make")) {
        (Ok(path), _) => Ok(path),
        (_, Ok(path)) => Ok(path),
        _ => Err(IoError::new(NotFound, "Unable to find make in path (gmake or make)")),
    }?;

    // Level of concurrency to use when building NGINX - cargo nicely provides this information
    let num_jobs = match env::var("NUM_JOBS") {
        Ok(s) => s.parse::<usize>().ok(),
        Err(_) => thread::available_parallelism().ok().map(|n| n.get()),
    }
    .unwrap_or(1);

    /* Use the duct dependency here to merge the output of STDOUT and STDERR into a single stream,
    and to provide the combined output as a reader which can be iterated over line-by-line. We
    use duct to do this because it is a lot of work to implement this from scratch. */
    cmd!(make_bin_path, "-j", num_jobs.to_string(), arg)
        .dir(nginx_src_dir)
        .stderr_to_stdout()
        .run()
}

/// Reads through the makefile generated by autoconf and finds all of the includes
/// used to compile NGINX. This is used to generate the correct bindings for the
/// NGINX source code.
fn parse_includes_from_makefile(nginx_autoconf_makefile_path: &PathBuf) -> Vec<PathBuf> {
    fn extract_include_part(line: &str) -> &str {
        line.strip_suffix('\\').map_or(line, |s| s.trim())
    }
    /// Extracts the include path from a line of the autoconf generated makefile.
    fn extract_after_i_flag(line: &str) -> Option<&str> {
        let mut parts = line.split("-I ");
        match parts.next() {
            Some(_) => parts.next().map(extract_include_part),
            None => None,
        }
    }

    let mut includes = vec![];
    let makefile_contents = match std::fs::read_to_string(nginx_autoconf_makefile_path) {
        Ok(path) => path,
        Err(e) => {
            panic!(
                "Unable to read makefile from path [{}]. Error: {}",
                nginx_autoconf_makefile_path.to_string_lossy(),
                e
            );
        }
    };

    let mut includes_lines = false;
    for line in makefile_contents.lines() {
        if !includes_lines {
            if let Some(stripped) = line.strip_prefix("ALL_INCS") {
                includes_lines = true;
                if let Some(part) = extract_after_i_flag(stripped) {
                    includes.push(part);
                }
                continue;
            }
        }

        if includes_lines {
            if let Some(part) = extract_after_i_flag(line) {
                includes.push(part);
            } else {
                break;
            }
        }
    }

    let makefile_dir = nginx_autoconf_makefile_path
        .parent()
        .expect("makefile path has no parent")
        .parent()
        .expect("objs dir has no parent")
        .to_path_buf()
        .canonicalize()
        .expect("Unable to canonicalize makefile path");

    includes
        .into_iter()
        .map(PathBuf::from)
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                makefile_dir.join(path)
            }
        })
        .collect()
}
