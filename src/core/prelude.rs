pub use std::ffi::{c_char, c_void};

pub use crate::ffi::ngx_command_t;
pub use crate::ffi::ngx_conf_t;
pub use crate::ffi::ngx_connection_t;
pub use crate::ffi::ngx_event_t;
pub use crate::ffi::ngx_int_t;
pub use crate::ffi::ngx_module_t;
pub use crate::ffi::ngx_str_t;
pub use crate::ffi::ngx_uint_t;
pub use crate::ffi::ngx_variable_value_t;

pub use crate::ffi::NGX_LOG_ALERT;
pub use crate::ffi::NGX_LOG_CRIT;
pub use crate::ffi::NGX_LOG_DEBUG;
pub use crate::ffi::NGX_LOG_EMERG;
pub use crate::ffi::NGX_LOG_ERR;
pub use crate::ffi::NGX_LOG_INFO;
pub use crate::ffi::NGX_LOG_NOTICE;
pub use crate::ffi::NGX_LOG_WARN;

pub use crate::ffi::NGX_ANY_CONF;
pub use crate::ffi::NGX_CONF_1MORE;
pub use crate::ffi::NGX_CONF_2MORE;
pub use crate::ffi::NGX_CONF_ANY;
pub use crate::ffi::NGX_CONF_ARGS_NUMBER;
pub use crate::ffi::NGX_CONF_BLOCK;
pub use crate::ffi::NGX_CONF_FLAG;
pub use crate::ffi::NGX_CONF_NOARGS;
pub use crate::ffi::NGX_CONF_TAKE1;
pub use crate::ffi::NGX_CONF_TAKE2;
pub use crate::ffi::NGX_CONF_TAKE3;
pub use crate::ffi::NGX_CONF_TAKE4;
pub use crate::ffi::NGX_CONF_TAKE5;
pub use crate::ffi::NGX_CONF_TAKE6;
pub use crate::ffi::NGX_CONF_TAKE7;
pub use crate::ffi::NGX_CONF_UNSET;
pub use crate::ffi::NGX_DIRECT_CONF;
pub use crate::ffi::NGX_MAIN_CONF;

pub use crate::ffi::ngx_cycle;
pub use crate::ffi::ngx_posted_events;

/// Default module value for the `ngx_module_t` struct.
pub const NGX_RS_MODULE_V1: ngx_module_t = ngx_module_t {
    ctx_index: ngx_uint_t::MAX,
    index: ngx_uint_t::MAX,
    name: std::ptr::null_mut(),
    spare0: 0,
    spare1: 0,
    version: crate::ffi::nginx_version as ngx_uint_t,
    signature: crate::ffi::NGX_RS_MODULE_SIGNATURE.as_ptr() as *const c_char,
    ctx: std::ptr::null_mut(),
    commands: std::ptr::null_mut(),
    type_: 0,
    init_master: None,
    init_module: None,
    init_process: None,
    init_thread: None,
    exit_thread: None,
    exit_process: None,
    exit_master: None,
    spare_hook0: 0,
    spare_hook1: 0,
    spare_hook2: 0,
    spare_hook3: 0,
    spare_hook4: 0,
    spare_hook5: 0,
    spare_hook6: 0,
    spare_hook7: 0,
};
