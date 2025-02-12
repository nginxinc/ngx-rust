mod buffer;
mod pool;
mod status;
mod string;

pub use buffer::*;
pub use pool::*;
pub use status::*;
pub use string::*;

/// Static empty configuration directive initializer for [`ngx_command_t`].
///
/// This is typically used to terminate an array of configuration directives.
///
/// [`ngx_command_t`]: https://nginx.org/en/docs/dev/development_guide.html#config_directives
#[macro_export]
macro_rules! ngx_null_command {
    () => {
        $crate::ffi::ngx_command_t {
            name: $crate::ngx_null_string!(),
            type_: 0,
            set: None,
            conf: 0,
            offset: 0,
            post: ::core::ptr::null_mut(),
        }
    };
}
