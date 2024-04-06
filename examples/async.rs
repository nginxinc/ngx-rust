use ngx::ffi::{
    nginx_version, ngx_array_push, ngx_command_t, ngx_conf_t, ngx_cycle, ngx_event_t, ngx_http_core_module,
    ngx_http_core_run_phases, ngx_http_handler_pt, ngx_http_module_t, ngx_http_phases_NGX_HTTP_ACCESS_PHASE,
    ngx_http_request_t, ngx_int_t, ngx_module_t, ngx_posted_events, ngx_queue_s, ngx_str_t, ngx_uint_t, NGX_CONF_TAKE1,
    NGX_HTTP_LOC_CONF, NGX_HTTP_MODULE, NGX_RS_HTTP_LOC_CONF_OFFSET, NGX_RS_MODULE_SIGNATURE,
};
use ngx::http::MergeConfigError;
use ngx::{core, core::Status, http, http::HTTPModule};
use ngx::{http_request_handler, ngx_log_debug_http, ngx_modules, ngx_null_command, ngx_string};
use std::borrow::Borrow;
use std::os::raw::{c_char, c_void};
use std::ptr::addr_of_mut;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::Runtime;

struct Module;

impl http::HTTPModule for Module {
    type MainConf = ();
    type SrvConf = ();
    type LocConf = ModuleConfig;

    unsafe extern "C" fn postconfiguration(cf: *mut ngx_conf_t) -> ngx_int_t {
        let cmcf = http::ngx_http_conf_get_module_main_conf(cf, ngx_http_core_module.borrow());

        let h = ngx_array_push(&mut (*cmcf).phases[ngx_http_phases_NGX_HTTP_ACCESS_PHASE as usize].handlers)
            as *mut ngx_http_handler_pt;
        if h.is_null() {
            return core::Status::NGX_ERROR.into();
        }
        // set an Access phase handler
        *h = Some(async_access_handler);
        core::Status::NGX_OK.into()
    }
}

#[derive(Debug)]
struct ModuleConfig {
    enable: bool,
    rt: Runtime,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            enable: Default::default(),
            rt: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
        }
    }
}

#[no_mangle]
static mut ngx_http_async_commands: [ngx_command_t; 2] = [
    ngx_command_t {
        name: ngx_string!("async"),
        type_: (NGX_HTTP_LOC_CONF | NGX_CONF_TAKE1) as ngx_uint_t,
        set: Some(ngx_http_async_commands_set_enable),
        conf: NGX_RS_HTTP_LOC_CONF_OFFSET,
        offset: 0,
        post: std::ptr::null_mut(),
    },
    ngx_null_command!(),
];

#[no_mangle]
static ngx_http_async_module_ctx: ngx_http_module_t = ngx_http_module_t {
    preconfiguration: Some(Module::preconfiguration),
    postconfiguration: Some(Module::postconfiguration),
    create_main_conf: Some(Module::create_main_conf),
    init_main_conf: Some(Module::init_main_conf),
    create_srv_conf: Some(Module::create_srv_conf),
    merge_srv_conf: Some(Module::merge_srv_conf),
    create_loc_conf: Some(Module::create_loc_conf),
    merge_loc_conf: Some(Module::merge_loc_conf),
};

ngx_modules!(ngx_http_async_module);

#[no_mangle]
pub static mut ngx_http_async_module: ngx_module_t = ngx_module_t {
    ctx_index: ngx_uint_t::max_value(),
    index: ngx_uint_t::max_value(),
    name: std::ptr::null_mut(),
    spare0: 0,
    spare1: 0,
    version: nginx_version as ngx_uint_t,
    signature: NGX_RS_MODULE_SIGNATURE.as_ptr() as *const c_char,

    ctx: &ngx_http_async_module_ctx as *const _ as *mut _,
    commands: unsafe { &ngx_http_async_commands[0] as *const _ as *mut _ },
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

impl http::Merge for ModuleConfig {
    fn merge(&mut self, prev: &ModuleConfig) -> Result<(), MergeConfigError> {
        if prev.enable {
            self.enable = true;
        };
        Ok(())
    }
}

unsafe extern "C" fn check_async_work_done(event: *mut ngx_event_t) {
    let event = &mut (*event);
    let data = Arc::from_raw(event.data as *const EventData);
    let req = &mut (*(data.request as *const _ as *mut ngx_http_request_t));
    if data.done_flag.load(std::sync::atomic::Ordering::Relaxed) {
        (*req.main).set_count((*req.main).count() - 1);
        ngx_http_core_run_phases(req);
    } else {
        // this doesn't have have good performance but works as a simple thread-safe example and doesn't causes
        // segfault. The best method that provides both thread-safety and performance requires
        // an nginx patch.
        post_event(event, addr_of_mut!(ngx_posted_events));
    }
}

struct RequestCTX {
    event_data: Option<Arc<EventData>>,
}

struct EventData {
    done_flag: AtomicBool,
    request: *mut ngx_http_request_t,
}

unsafe impl Send for EventData {}
unsafe impl Sync for EventData {}

// same as ngx_post_event
// source: https://github.com/nginxinc/ngx-rust/pull/31/files#diff-132330bb775bed17fb9990ec2b56e6c52e6a9e56d62f2114fade95e4decdba08R80-R90
unsafe fn post_event(event: *mut ngx_event_t, queue: *mut ngx_queue_s) {
    let event = &mut (*event);
    if event.posted() == 0 {
        event.set_posted(1);
        // translated from ngx_queue_insert_tail macro
        event.queue.prev = (*queue).prev;
        (*event.queue.prev).next = &event.queue as *const _ as *mut _;
        event.queue.next = queue;
        (*queue).prev = &event.queue as *const _ as *mut _;
    }
}

http_request_handler!(async_access_handler, |request: &mut http::Request| {
    let co = unsafe { request.get_module_loc_conf::<ModuleConfig>(ngx_http_async_module.borrow()) };
    let co = co.expect("module config is none");
    if !co.enable {
        return core::Status::NGX_DECLINED;
    }

    let event_data = unsafe {
        let ctx = request.get_inner().ctx.add(ngx_http_async_module.ctx_index);
        if (*ctx).is_null() {
            let ctx_data = &mut *(request.pool().alloc(std::mem::size_of::<RequestCTX>()) as *mut RequestCTX);
            ctx_data.event_data = Some(Arc::new(EventData {
                done_flag: AtomicBool::new(false),
                request: &request.get_inner() as *const _ as *mut _,
            }));
            *ctx = ctx_data as *const _ as _;
            ctx_data.event_data.as_ref().unwrap().clone()
        } else {
            let ctx = &*(ctx as *const RequestCTX);
            if ctx
                .event_data
                .as_ref()
                .unwrap()
                .done_flag
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                return core::Status::NGX_OK;
            } else {
                return core::Status::NGX_DONE;
            }
        }
    };

    event_data.done_flag.load(std::sync::atomic::Ordering::Relaxed);

    // create a posted event
    unsafe {
        let event = &mut *(request.pool().calloc(std::mem::size_of::<ngx_event_t>()) as *mut ngx_event_t);
        event.handler = Some(check_async_work_done);
        event.data = Arc::into_raw(event_data.clone()) as _;
        event.log = (*ngx_cycle).log;

        post_event(event, addr_of_mut!(ngx_posted_events));
    }

    ngx_log_debug_http!(request, "async module enabled: {}", co.enable);

    co.rt.spawn(async move {
        let start = Instant::now();
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let req = unsafe { http::Request::from_ngx_http_request(event_data.request) };
        // not really thread safe, we should apply all these operation in nginx thread
        // but this is just an example. proper way would be storing these headers in the request ctx
        // and apply them when we get back to the nginx thread.
        req.add_header_out("X-Async-Time", start.elapsed().as_millis().to_string().as_str());

        event_data.done_flag.store(true, std::sync::atomic::Ordering::Release);
        // there is a small issue here. If traffic is low we may get stuck behind a 300ms timer
        // in the nginx event loop. To workaround it we can notify the event loop using pthread_kill( nginx_thread, SIGIO )
        // to wake up the event loop. (or patch nginx and use the same trick as the thread pool)
    });

    unsafe {
        (*request.get_inner().main).set_count((*request.get_inner().main).count() + 1);
    }
    core::Status::NGX_DONE
});

#[no_mangle]
extern "C" fn ngx_http_async_commands_set_enable(
    cf: *mut ngx_conf_t,
    _cmd: *mut ngx_command_t,
    conf: *mut c_void,
) -> *mut c_char {
    unsafe {
        let conf = &mut *(conf as *mut ModuleConfig);
        let args = (*(*cf).args).elts as *mut ngx_str_t;

        let val = (*args.add(1)).to_str();

        // set default value optionally
        conf.enable = false;

        if val.len() == 2 && val.eq_ignore_ascii_case("on") {
            conf.enable = true;
        } else if val.len() == 3 && val.eq_ignore_ascii_case("off") {
            conf.enable = false;
        }
    };

    std::ptr::null_mut()
}
