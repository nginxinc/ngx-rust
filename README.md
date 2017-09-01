# Rust Wrapper for Nginx

Please follow steps

## Install Rust tooling

Follow instruction here to install rust tooling

https://www.rust-lang.org/en-US/install.html

## Configure Rust module

This project need to be configured for specific OS in order to build other module which depends on this module.

This will generate 'nginx' directory which contains configured version of nginx for each of the OS.

### For Linux development

If you want to target your module to run on linux, please perform one-time setup.  This will download nginx source and configure to compile on linux.

```bash
make linux-setup
```

### For Mac development

```bash
make darwin-setup
```

## Clean configuration

This will remove all Rust artifacts as well generated NGINX build artifacts
```bash
make super_clean
```
 




