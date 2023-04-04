use crate::ffi::*;

use std::os::raw::c_void;

/// # Safety
///
/// The caller has provided a valid `ngx_conf_t` that points to valid memory and is non-null.
pub unsafe fn ngx_http_conf_get_module_main_conf(
    cf: *mut ngx_conf_t,
    module: &ngx_module_t,
) -> *mut ngx_http_core_main_conf_t {
    let http_conf_ctx = (*cf).ctx as *mut ngx_http_conf_ctx_t;
    *(*http_conf_ctx).main_conf.add(module.ctx_index) as *mut ngx_http_core_main_conf_t
}

/// # Safety
///
/// The caller has provided a valid `ngx_conf_t` that points to valid memory and is non-null.
pub unsafe fn ngx_http_conf_get_module_srv_conf(cf: *mut ngx_conf_t, module: &ngx_module_t) -> *mut c_void {
    let http_conf_ctx = (*cf).ctx as *mut ngx_http_conf_ctx_t;
    *(*http_conf_ctx).srv_conf.add(module.ctx_index)
}

/// # Safety
///
/// The caller has provided a valid `ngx_conf_t` that points to valid memory and is non-null.
pub unsafe fn ngx_http_conf_get_module_loc_conf(
    cf: *mut ngx_conf_t,
    module: &ngx_module_t,
) -> *mut ngx_http_core_loc_conf_t {
    let http_conf_ctx = (*cf).ctx as *mut ngx_http_conf_ctx_t;
    *(*http_conf_ctx).loc_conf.add(module.ctx_index) as *mut ngx_http_core_loc_conf_t
}
