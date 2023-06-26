/*
 * This example is based on:
 * https://github.com/gabihodoroaga/nginx-upstream-module
 *
 * The NGINX authors are grateful to @gabihodoroaga for their contributions
 * to the community at large.
 * https://github.com/gabihodoroaga
 */
use ngx::{
    core::{Pool, Status},
    ffi::{
        nginx_version, ngx_atoi, ngx_command_t, ngx_conf_log_error, ngx_conf_t, ngx_connection_t,
        ngx_event_free_peer_pt, ngx_event_get_peer_pt, ngx_http_module_t, ngx_http_request_t,
        ngx_http_upstream_init_peer_pt, ngx_http_upstream_init_pt, ngx_http_upstream_init_round_robin,
        ngx_http_upstream_srv_conf_t, ngx_http_upstream_t, ngx_int_t, ngx_module_t, ngx_peer_connection_t, ngx_str_t,
        ngx_uint_t, NGX_CONF_NOARGS, NGX_CONF_TAKE1, NGX_CONF_UNSET, NGX_ERROR, NGX_HTTP_MODULE, NGX_HTTP_UPS_CONF,
        NGX_LOG_DEBUG_HTTP, NGX_LOG_EMERG, NGX_RS_HTTP_SRV_CONF_OFFSET, NGX_RS_MODULE_SIGNATURE,
    },
    http::{ngx_http_conf_get_module_srv_conf, HTTPModule, Merge, MergeConfigError, Request},
    ngx_log_debug_http, ngx_log_debug_mask, ngx_modules, ngx_null_command, ngx_string,
};
use std::{
    mem,
    os::raw::{c_char, c_void},
    slice,
};

//FIXME move this to src/http/request.rs or an upstream.rs?
#[macro_export]
macro_rules! http_upstream_peer_init {
    ( $name: ident, $handler: expr ) => {
        #[no_mangle]
        unsafe extern "C" fn $name(r: *mut ngx_http_request_t, us: *mut ngx_http_upstream_srv_conf_t) -> ngx_int_t {
            let status: Status = $handler(unsafe { &mut Request::from_ngx_http_request(r) }, us);
            status.0
        }
    };
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct SrvConfig {
    max: u32,

    //FIXME: should these be traits that a server implements to make the
    //functions easier to use?
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
    upstream: *mut ngx_http_upstream_t,
    data: *mut c_void,
    client_connection: *mut ngx_connection_t,
    original_get_peer: ngx_event_get_peer_pt,
    original_free_peer: ngx_event_free_peer_pt,
}

impl Default for UpstreamPeerData {
    fn default() -> Self {
        UpstreamPeerData {
            conf: None,
            upstream: std::ptr::null_mut(),
            data: std::ptr::null_mut(),
            client_connection: std::ptr::null_mut(),
            original_get_peer: None,
            original_free_peer: None,
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

http_upstream_peer_init!(
    init_custom_peer,
    |request: &mut Request, us: *mut ngx_http_upstream_srv_conf_t| {
        ngx_log_debug_http!(request, "custom init peer");

        let mut hcpd = request.pool().alloc_type::<UpstreamPeerData>();
        if hcpd.is_null() {
            return Status::NGX_ERROR;
        }

        // FIXME make this a convenience macro?
        let hccf: *const SrvConfig = (*us)
            .srv_conf
            .offset(ngx_http_upstream_custom_module.ctx_index as isize)
            as *const SrvConfig;

        // FIXME method or macro?
        // Casting from Rust types to C function pointers might be better as macros for
        // original_init_peer, free, and get.
        //
        // Casting to ngx_http_request_t might also benefit. Alternatively a trait which config and
        // upstream data use may work as well.
        let original_init_peer: unsafe extern "C" fn(
            *mut ngx_http_request_t,
            *mut ngx_http_upstream_srv_conf_t,
        ) -> ngx_int_t = unsafe { mem::transmute((*hccf).original_init_peer) };

        {
            let r: *mut ngx_http_request_t = unsafe { mem::transmute(&request) };
            if original_init_peer(r, us) != Status::NGX_OK.into() {
                return Status::NGX_ERROR;
            }
        }

        let upstream_ptr = request.upstream();

        (*hcpd).conf = Some(hccf);
        (*hcpd).upstream = upstream_ptr;
        (*hcpd).data = (*upstream_ptr).peer.data;
        (*hcpd).client_connection = request.connection();
        (*hcpd).original_get_peer = (*upstream_ptr).peer.get;
        (*hcpd).original_free_peer = (*upstream_ptr).peer.free;

        (*upstream_ptr).peer.data = hcpd as *mut c_void;
        (*upstream_ptr).peer.get = Some(ngx_http_upstream_get_custom_peer);
        (*upstream_ptr).peer.free = Some(ngx_http_upstream_free_custom_peer);

        Status::NGX_OK
    }
);

#[no_mangle]
unsafe extern "C" fn ngx_http_upstream_get_custom_peer(pc: *mut ngx_peer_connection_t, data: *mut c_void) -> ngx_int_t {
    let hcdp: *mut UpstreamPeerData = unsafe { mem::transmute(data) };

    //FIXME log

    let original_get_peer: unsafe extern "C" fn(*mut ngx_peer_connection_t, *mut c_void) -> ngx_int_t =
        unsafe { mem::transmute((*hcdp).original_get_peer) };
    let rc = original_get_peer(pc, (*hcdp).data);

    if rc != Status::NGX_OK.into() {
        return rc;
    }
    Status::NGX_OK.into()
}

#[no_mangle]
unsafe extern "C" fn ngx_http_upstream_free_custom_peer(
    pc: *mut ngx_peer_connection_t,
    data: *mut c_void,
    state: ngx_uint_t,
) {
    let hcdp: *mut UpstreamPeerData = unsafe { mem::transmute(data) };

    let original_free_peer: unsafe extern "C" fn(*mut ngx_peer_connection_t, data: *mut c_void, ngx_uint_t) =
        unsafe { mem::transmute((*hcdp).original_free_peer) };

    //FIXME log
    original_free_peer(pc, (*hcdp).data, state);
}

#[no_mangle]
unsafe extern "C" fn ngx_http_upstream_init_custom(
    cf: *mut ngx_conf_t,
    us: *mut ngx_http_upstream_srv_conf_t,
) -> ngx_int_t {
    ngx_log_debug_mask!(NGX_LOG_DEBUG_HTTP, (*cf).log, "custom init upstream");

    // FIXME: this comes from ngx_http_conf_upstream_srv_conf macro which isn't built into bindings
    // start creating a macros file?
    let hccf: *mut SrvConfig = (*us)
        .srv_conf
        .offset(ngx_http_upstream_custom_module.ctx_index as isize) as *mut SrvConfig;

    // FIXME: ngx_conf_init_uint_value macro is unavailable
    if (*hccf).max == u32::MAX {
        (*hccf).max = 100;
    }

    //FIXME make a trait and call this as a method on SrvConfig?
    let init_upstream_ptr: unsafe extern "C" fn(*mut ngx_conf_t, *mut ngx_http_upstream_srv_conf_t) -> ngx_int_t =
        unsafe { mem::transmute((*hccf).original_init_upstream) };
    if init_upstream_ptr(cf, us) != Status::NGX_OK.into() {
        return isize::from(Status::NGX_ERROR);
    }

    (*hccf).original_init_peer = (*us).peer.init;
    //(*us).peer.init = Some(ngx_http_upstream_init_custom_peer);
    (*us).peer.init = Some(init_custom_peer);

    isize::from(Status::NGX_OK)
}

#[no_mangle]
unsafe extern "C" fn ngx_http_upstream_commands_set_custom(
    cf: *mut ngx_conf_t,
    cmd: *mut ngx_command_t,
    conf: *mut c_void,
) -> *mut c_char {
    //FIXME need a log macros that accepts level and masks:
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

    let uscf: *mut ngx_http_upstream_srv_conf_t =
        ngx_http_conf_get_module_srv_conf(cf, &ngx_http_upstream_custom_module) as *mut ngx_http_upstream_srv_conf_t;

    ccf.original_init_upstream = if (*uscf).peer.init_upstream.is_some() {
        (*uscf).peer.init_upstream
    } else {
        Some(ngx_http_upstream_init_round_robin)
    };

    (*uscf).peer.init_upstream = Some(ngx_http_upstream_init_custom);

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
