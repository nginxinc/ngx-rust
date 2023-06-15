use ngx::{
    core::Pool,
    ffi::{
        nginx_version, ngx_atoi, ngx_command_t, ngx_conf_log_error, ngx_conf_t, ngx_http_module_t,
        ngx_http_upstream_init_peer_pt, ngx_http_upstream_init_pt, ngx_module_t, ngx_str_t, ngx_uint_t,
        NGX_CONF_NOARGS, NGX_CONF_TAKE1, NGX_CONF_UNSET, NGX_ERROR, NGX_HTTP_MODULE, NGX_HTTP_UPS_CONF, NGX_LOG_EMERG,
        NGX_RS_HTTP_SRV_CONF_OFFSET, NGX_RS_MODULE_SIGNATURE,
    },
    http::{ngx_http_conf_get_module_srv_conf, HTTPModule, Merge, MergeConfigError},
    ngx_modules, ngx_null_command, ngx_string,
};
use std::{
    os::raw::{c_char, c_void},
    slice,
};

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
struct SrvConfig {
    max: u32,
    original_init_upstream: ngx_http_upstream_init_pt,
    origin_init_peer: ngx_http_upstream_init_peer_pt,
}

impl Merge for SrvConfig {
    fn merge(&mut self, _prev: &SrvConfig) -> Result<(), MergeConfigError> {
        Ok(())
    }
}

#[no_mangle]
static ngx_http_upstream_custom_ctx: ngx_http_module_t = ngx_http_module_t {
    preconfiguration: Some(Module::preconfiguration),
    postconfiguration: Some(Module::postconfiguration),
    create_main_conf: Some(Module::create_main_conf),
    init_main_conf: Some(Module::init_main_conf),
    create_srv_conf: Some(Module::create_srv_conf),
    merge_srv_conf: Some(Module::merge_srv_conf),
    create_loc_conf: Some(Module::create_loc_conf),
    merge_loc_conf: Some(Module::merge_loc_conf),
};

#[no_mangle]
static mut ngx_http_upstream_custom_commands: [ngx_command_t; 2] = [
    ngx_command_t {
        name: ngx_string!("custom"),
        type_: (NGX_HTTP_UPS_CONF | NGX_CONF_NOARGS | NGX_CONF_TAKE1) as ngx_uint_t,
        set: Some(ngx_http_upstream_commands_set_custom),
        conf: NGX_RS_HTTP_SRV_CONF_OFFSET,
        offset: 0,
        post: std::ptr::null_mut(),
    },
    ngx_null_command!(),
];

ngx_modules!(ngx_http_upstream_custom_module);

#[no_mangle]
pub static mut ngx_http_upstream_custom_module: ngx_module_t = ngx_module_t {
    ctx_index: ngx_uint_t::max_value(),
    index: ngx_uint_t::max_value(),
    name: std::ptr::null_mut(),
    spare0: 0,
    spare1: 0,
    version: nginx_version as ngx_uint_t,
    signature: NGX_RS_MODULE_SIGNATURE.as_ptr() as *const c_char,

    ctx: &ngx_http_upstream_custom_ctx as *const _ as *mut _,
    commands: unsafe { &ngx_http_upstream_custom_commands[0] as *const _ as *mut _ },
    type_: NGX_HTTP_MODULE as ngx_uint_t,

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

#[no_mangle]
unsafe extern "C" fn ngx_http_upstream_commands_set_custom(
    cf: *mut ngx_conf_t,
    cmd: *mut ngx_command_t,
    conf: *mut c_void,
) -> *mut c_char {
    //TODO need a log macros that accepts level and masks:
    //  NGX_LOG_DEBUG_HTTP, NGX_LOG_DEBUG_EVENT, etc.

    let mut ccf = &mut (*(conf as *mut SrvConfig));

    if (*(*cf).args).nelts == 2 {
        let value: &[ngx_str_t] = slice::from_raw_parts((*(*cf).args).elts as *const ngx_str_t, (*(*cf).args).nelts);
        let n = ngx_atoi(value[1].data, value[1].len);
        if n == (NGX_ERROR as isize) || n == 0 {
            ngx_conf_log_error(
                NGX_LOG_EMERG as usize,
                cf,
                0,
                "invalid value \"%V\" in \"%V\" directive".as_bytes().as_ptr() as *const i8,
                value[1],
                &(*cmd).name,
            );
            return usize::MAX as *mut i8;
        }
        ccf.max = n as u32;
    }

    let uscf = ngx_http_conf_get_module_srv_conf(cf, &ngx_http_upstream_custom_module);

    //ccf.original_init_upstream = if (*upstream_conf).peer.init_upstream.is_null() {

    // NGX_CONF_OK
    std::ptr::null_mut()
}

struct Module;

impl HTTPModule for Module {
    type MainConf = ();
    type SrvConf = SrvConfig;
    type LocConf = ();

    unsafe extern "C" fn create_srv_conf(cf: *mut ngx_conf_t) -> *mut c_void {
        let mut pool = Pool::from_ngx_pool((*cf).pool);
        let conf = pool.alloc_type::<SrvConfig>();
        if conf.is_null() {
            return std::ptr::null_mut();
        }

        (*conf).max = NGX_CONF_UNSET as u32;

        conf as *mut c_void
    }
}
