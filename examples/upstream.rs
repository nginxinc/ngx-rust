/*
 * This example is based on:
 * https://github.com/gabihodoroaga/nginx-upstream-module
 * as well as the NGINX keepalive module: ngx_http_upstream_keepalive_module.c.
 *
 * The NGINX authors are grateful to @gabihodoroaga for their contributions
 * to the community at large.
 */
use ngx::{
    core::{Pool, Status},
    ffi::{
        nginx_version, ngx_atoi, ngx_command_t, ngx_conf_log_error, ngx_conf_t, ngx_connection_t,
        ngx_event_free_peer_pt, ngx_event_get_peer_pt, ngx_http_module_t, ngx_http_request_t,
        ngx_http_upstream_init_peer_pt, ngx_http_upstream_init_pt, ngx_http_upstream_init_round_robin,
        ngx_http_upstream_module, ngx_http_upstream_srv_conf_t, ngx_http_upstream_t, ngx_int_t, ngx_module_t,
        ngx_peer_connection_t, ngx_str_t, ngx_uint_t, NGX_CONF_NOARGS, NGX_CONF_TAKE1, NGX_CONF_UNSET, NGX_ERROR,
        NGX_HTTP_MODULE, NGX_HTTP_UPS_CONF, NGX_LOG_EMERG, NGX_RS_HTTP_SRV_CONF_OFFSET, NGX_RS_MODULE_SIGNATURE,
    },
    http::{
        ngx_http_conf_get_module_srv_conf, ngx_http_conf_upstream_srv_conf_immutable,
        ngx_http_conf_upstream_srv_conf_mutable, HTTPModule, Merge, MergeConfigError, Request,
    },
    http_upstream_init_peer_pt,
    log::DebugMask,
    ngx_log_debug_http, ngx_log_debug_mask, ngx_modules, ngx_null_command, ngx_string,
};
use std::{
    mem,
    os::raw::{c_char, c_void},
    ptr::addr_of,
    slice,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct SrvConfig {
    max: u32,

    original_init_upstream: ngx_http_upstream_init_pt,
    original_init_peer: ngx_http_upstream_init_peer_pt,
}

impl Default for SrvConfig {
    fn default() -> Self {
        SrvConfig {
            max: u32::MAX,
            original_init_upstream: None,
            original_init_peer: None,
        }
    }
}

impl Merge for SrvConfig {
    fn merge(&mut self, _prev: &SrvConfig) -> Result<(), MergeConfigError> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct UpstreamPeerData {
    conf: Option<*const SrvConfig>,
    upstream: Option<*mut ngx_http_upstream_t>,
    client_connection: Option<*mut ngx_connection_t>,
    original_get_peer: ngx_event_get_peer_pt,
    original_free_peer: ngx_event_free_peer_pt,
    data: *mut c_void,
}

impl Default for UpstreamPeerData {
    fn default() -> Self {
        UpstreamPeerData {
            conf: None,
            upstream: None,
            client_connection: None,
            original_get_peer: None,
            original_free_peer: None,
            data: std::ptr::null_mut(),
        }
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

// http_upstream_init_custom_peer
// The module's custom peer.init callback. On HTTP request the peer upstream get and free callbacks
// are saved into peer data and replaced with this module's custom callbacks.
http_upstream_init_peer_pt!(
    http_upstream_init_custom_peer,
    |request: &mut Request, us: *mut ngx_http_upstream_srv_conf_t| {
        ngx_log_debug_http!(request, "CUSTOM UPSTREAM request peer init");

        let hcpd = request.pool().alloc_type::<UpstreamPeerData>();
        if hcpd.is_null() {
            return Status::NGX_ERROR;
        }

        let maybe_conf: Option<*const SrvConfig> =
            unsafe { ngx_http_conf_upstream_srv_conf_immutable(us, &*addr_of!(ngx_http_upstream_custom_module)) };
        if maybe_conf.is_none() {
            return Status::NGX_ERROR;
        }

        let hccf = maybe_conf.unwrap();
        let original_init_peer = unsafe { (*hccf).original_init_peer.unwrap() };
        if unsafe { original_init_peer(request.into(), us) != Status::NGX_OK.into() } {
            return Status::NGX_ERROR;
        }

        let maybe_upstream = request.upstream();
        if maybe_upstream.is_none() {
            return Status::NGX_ERROR;
        }
        let upstream_ptr = maybe_upstream.unwrap();

        unsafe {
            (*hcpd).conf = Some(hccf);
            (*hcpd).upstream = maybe_upstream;
            (*hcpd).data = (*upstream_ptr).peer.data;
            (*hcpd).client_connection = Some(request.connection());
            (*hcpd).original_get_peer = (*upstream_ptr).peer.get;
            (*hcpd).original_free_peer = (*upstream_ptr).peer.free;

            (*upstream_ptr).peer.data = hcpd as *mut c_void;
            (*upstream_ptr).peer.get = Some(ngx_http_upstream_get_custom_peer);
            (*upstream_ptr).peer.free = Some(ngx_http_upstream_free_custom_peer);
        }

        ngx_log_debug_http!(request, "CUSTOM UPSTREAM end request peer init");
        Status::NGX_OK
    }
);

// ngx_http_usptream_get_custom_peer
// For demonstration purposes, use the original get callback, but log this callback proxies through
// to the original.
#[no_mangle]
unsafe extern "C" fn ngx_http_upstream_get_custom_peer(pc: *mut ngx_peer_connection_t, data: *mut c_void) -> ngx_int_t {
    let hcpd: *mut UpstreamPeerData = unsafe { mem::transmute(data) };

    ngx_log_debug_mask!(
        DebugMask::Http,
        (*pc).log,
        "CUSTOM UPSTREAM get peer, try: {}, conn: {:p}",
        (*pc).tries,
        (*hcpd).client_connection.unwrap(),
    );

    let original_get_peer = (*hcpd).original_get_peer.unwrap();
    let rc = original_get_peer(pc, (*hcpd).data);

    if rc != Status::NGX_OK.into() {
        return rc;
    }

    ngx_log_debug_mask!(DebugMask::Http, (*pc).log, "CUSTOM UPSTREAM end get peer");
    Status::NGX_OK.into()
}

// ngx_http_upstream_free_custom_peer
// For demonstration purposes, use the original free callback, but log this callback proxies
// through to the original.
#[no_mangle]
unsafe extern "C" fn ngx_http_upstream_free_custom_peer(
    pc: *mut ngx_peer_connection_t,
    data: *mut c_void,
    state: ngx_uint_t,
) {
    ngx_log_debug_mask!(DebugMask::Http, (*pc).log, "CUSTOM UPSTREAM free peer");

    let hcpd: *mut UpstreamPeerData = unsafe { mem::transmute(data) };

    let original_free_peer = (*hcpd).original_free_peer.unwrap();

    original_free_peer(pc, (*hcpd).data, state);

    ngx_log_debug_mask!(DebugMask::Http, (*pc).log, "CUSTOM UPSTREAM end free peer");
}

// ngx_http_upstream_init_custom
// The module's custom `peer.init_upstream` callback.
// The original callback is saved in our SrvConfig data and reset to this module's `peer.init`.
#[no_mangle]
unsafe extern "C" fn ngx_http_upstream_init_custom(
    cf: *mut ngx_conf_t,
    us: *mut ngx_http_upstream_srv_conf_t,
) -> ngx_int_t {
    ngx_log_debug_mask!(DebugMask::Http, (*cf).log, "CUSTOM UPSTREAM peer init_upstream");

    let maybe_conf: Option<*mut SrvConfig> =
        ngx_http_conf_upstream_srv_conf_mutable(us, &*addr_of!(ngx_http_upstream_custom_module));
    if maybe_conf.is_none() {
        ngx_conf_log_error(
            NGX_LOG_EMERG as usize,
            cf,
            0,
            "CUSTOM UPSTREAM no upstream srv_conf".as_bytes().as_ptr() as *const c_char,
        );
        return isize::from(Status::NGX_ERROR);
    }
    let hccf = maybe_conf.unwrap();
    // NOTE: ngx_conf_init_uint_value macro is unavailable
    if (*hccf).max == u32::MAX {
        (*hccf).max = 100;
    }

    let init_upstream_ptr = (*hccf).original_init_upstream.unwrap();
    if init_upstream_ptr(cf, us) != Status::NGX_OK.into() {
        ngx_conf_log_error(
            NGX_LOG_EMERG as usize,
            cf,
            0,
            "CUSTOM UPSTREAM failed calling init_upstream".as_bytes().as_ptr() as *const c_char,
        );
        return isize::from(Status::NGX_ERROR);
    }

    (*hccf).original_init_peer = (*us).peer.init;
    (*us).peer.init = Some(http_upstream_init_custom_peer);

    ngx_log_debug_mask!(DebugMask::Http, (*cf).log, "CUSTOM UPSTREAM end peer init_upstream");
    isize::from(Status::NGX_OK)
}

// ngx_http_upstream_commands_set_custom
// Entry point for the module, if this command is set our custom upstreams take effect.
// The original upstream initializer function is saved and replaced with this module's initializer.
#[no_mangle]
unsafe extern "C" fn ngx_http_upstream_commands_set_custom(
    cf: *mut ngx_conf_t,
    cmd: *mut ngx_command_t,
    conf: *mut c_void,
) -> *mut c_char {
    ngx_log_debug_mask!(DebugMask::Http, (*cf).log, "CUSTOM UPSTREAM module init");

    let ccf = &mut (*(conf as *mut SrvConfig));

    if (*(*cf).args).nelts == 2 {
        let value: &[ngx_str_t] = slice::from_raw_parts((*(*cf).args).elts as *const ngx_str_t, (*(*cf).args).nelts);
        let n = ngx_atoi(value[1].data, value[1].len);
        if n == (NGX_ERROR as isize) || n == 0 {
            ngx_conf_log_error(
                NGX_LOG_EMERG as usize,
                cf,
                0,
                "invalid value \"%V\" in \"%V\" directive".as_bytes().as_ptr() as *const c_char,
                value[1],
                &(*cmd).name,
            );
            return usize::MAX as *mut c_char;
        }
        ccf.max = n as u32;
    }

    let uscf: *mut ngx_http_upstream_srv_conf_t =
        ngx_http_conf_get_module_srv_conf(cf, &*addr_of!(ngx_http_upstream_module))
            as *mut ngx_http_upstream_srv_conf_t;

    ccf.original_init_upstream = if (*uscf).peer.init_upstream.is_some() {
        (*uscf).peer.init_upstream
    } else {
        Some(ngx_http_upstream_init_round_robin)
    };

    (*uscf).peer.init_upstream = Some(ngx_http_upstream_init_custom);

    ngx_log_debug_mask!(DebugMask::Http, (*cf).log, "CUSTOM UPSTREAM end module init");
    // NGX_CONF_OK
    std::ptr::null_mut()
}

// The upstream module.
// Only server blocks are supported to trigger the module command; therefore, the only callback
// implemented is our `create_srv_conf` method.
struct Module;

impl HTTPModule for Module {
    type MainConf = ();
    type SrvConf = SrvConfig;
    type LocConf = ();

    unsafe extern "C" fn create_srv_conf(cf: *mut ngx_conf_t) -> *mut c_void {
        let mut pool = Pool::from_ngx_pool((*cf).pool);
        let conf = pool.alloc_type::<SrvConfig>();
        if conf.is_null() {
            ngx_conf_log_error(
                NGX_LOG_EMERG as usize,
                cf,
                0,
                "CUSTOM UPSTREAM could not allocate memory for config"
                    .as_bytes()
                    .as_ptr() as *const c_char,
            );
            return std::ptr::null_mut();
        }

        (*conf).max = NGX_CONF_UNSET as u32;

        ngx_log_debug_mask!(DebugMask::Http, (*cf).log, "CUSTOM UPSTREAM end create_srv_conf");
        conf as *mut c_void
    }
}
