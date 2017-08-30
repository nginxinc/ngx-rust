# Rust Nginx Rust wrapper

## Install Rust

Follow instruction here to install rust tooling

https://www.rust-lang.org/en-US/install.html

## Configure Rust module

For specific target OS, this project need to be configured in order to compiler other modules which are depend on this project.  
The project configured to support multiple OS.

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

This will remove existing configuration

```bash
make super_clean
```
 




