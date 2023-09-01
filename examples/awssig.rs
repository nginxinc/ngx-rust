use http::HeaderMap;
use ngx::ffi::{
    nginx_version, ngx_array_push, ngx_command_t, ngx_conf_t, ngx_http_core_module, ngx_http_handler_pt,
    ngx_http_module_t, ngx_http_phases_NGX_HTTP_PRECONTENT_PHASE, ngx_http_request_t, ngx_int_t, ngx_module_t,
    ngx_str_t, ngx_uint_t, NGX_CONF_TAKE1, NGX_HTTP_LOC_CONF, NGX_HTTP_MODULE, NGX_HTTP_SRV_CONF,
    NGX_RS_HTTP_LOC_CONF_OFFSET, NGX_RS_MODULE_SIGNATURE,
};
use ngx::{core, core::Status, http::*};
use ngx::{http_request_handler, ngx_log_debug_http, ngx_modules, ngx_null_command, ngx_string};
use std::os::raw::{c_char, c_void};

struct Module;

impl HTTPModule for Module {
    type MainConf = ();
    type SrvConf = ();
    type LocConf = ModuleConfig;

    unsafe extern "C" fn postconfiguration(cf: *mut ngx_conf_t) -> ngx_int_t {
        let cmcf = ngx_http_conf_get_module_main_conf_mut_ptr(cf, &ngx_http_core_module);

        let h = ngx_array_push(&mut (*cmcf).phases[ngx_http_phases_NGX_HTTP_PRECONTENT_PHASE as usize].handlers)
            as *mut ngx_http_handler_pt;
        if h.is_null() {
            return core::Status::NGX_ERROR.into();
        }
        // set an phase handler
        *h = Some(awssigv4_header_handler);
        core::Status::NGX_OK.into()
    }
}

#[derive(Debug, Default)]
struct ModuleConfig {
    enable: bool,
    access_key: String,
    secret_key: String,
    s3_bucket: String,
    s3_endpoint: String,
}

#[no_mangle]
static mut ngx_http_awssigv4_commands: [ngx_command_t; 6] = [
    ngx_command_t {
        name: ngx_string!("awssigv4"),
        type_: (NGX_HTTP_LOC_CONF | NGX_HTTP_SRV_CONF | NGX_CONF_TAKE1) as ngx_uint_t,
        set: Some(ngx_http_awssigv4_commands_set_enable),
        conf: NGX_RS_HTTP_LOC_CONF_OFFSET,
        offset: 0,
        post: std::ptr::null_mut(),
    },
    ngx_command_t {
        name: ngx_string!("awssigv4_access_key"),
        type_: (NGX_HTTP_LOC_CONF | NGX_HTTP_SRV_CONF | NGX_CONF_TAKE1) as ngx_uint_t,
        set: Some(ngx_http_awssigv4_commands_set_access_key),
        conf: NGX_RS_HTTP_LOC_CONF_OFFSET,
        offset: 0,
        post: std::ptr::null_mut(),
    },
    ngx_command_t {
        name: ngx_string!("awssigv4_secret_key"),
        type_: (NGX_HTTP_LOC_CONF | NGX_HTTP_SRV_CONF | NGX_CONF_TAKE1) as ngx_uint_t,
        set: Some(ngx_http_awssigv4_commands_set_secret_key),
        conf: NGX_RS_HTTP_LOC_CONF_OFFSET,
        offset: 0,
        post: std::ptr::null_mut(),
    },
    ngx_command_t {
        name: ngx_string!("awssigv4_s3_bucket"),
        type_: (NGX_HTTP_LOC_CONF | NGX_HTTP_SRV_CONF | NGX_CONF_TAKE1) as ngx_uint_t,
        set: Some(ngx_http_awssigv4_commands_set_s3_bucket),
        conf: NGX_RS_HTTP_LOC_CONF_OFFSET,
        offset: 0,
        post: std::ptr::null_mut(),
    },
    ngx_command_t {
        name: ngx_string!("awssigv4_s3_endpoint"),
        type_: (NGX_HTTP_LOC_CONF | NGX_HTTP_SRV_CONF | NGX_CONF_TAKE1) as ngx_uint_t,
        set: Some(ngx_http_awssigv4_commands_set_s3_endpoint),
        conf: NGX_RS_HTTP_LOC_CONF_OFFSET,
        offset: 0,
        post: std::ptr::null_mut(),
    },
    ngx_null_command!(),
];

#[no_mangle]
static ngx_http_awssigv4_module_ctx: ngx_http_module_t = ngx_http_module_t {
    preconfiguration: Some(Module::preconfiguration),
    postconfiguration: Some(Module::postconfiguration),
    create_main_conf: Some(Module::create_main_conf),
    init_main_conf: Some(Module::init_main_conf),
    create_srv_conf: Some(Module::create_srv_conf),
    merge_srv_conf: Some(Module::merge_srv_conf),
    create_loc_conf: Some(Module::create_loc_conf),
    merge_loc_conf: Some(Module::merge_loc_conf),
};

ngx_modules!(ngx_http_awssigv4_module);

#[no_mangle]
pub static mut ngx_http_awssigv4_module: ngx_module_t = ngx_module_t {
    ctx_index: ngx_uint_t::max_value(),
    index: ngx_uint_t::max_value(),
    name: std::ptr::null_mut(),
    spare0: 0,
    spare1: 0,
    version: nginx_version as ngx_uint_t,
    signature: NGX_RS_MODULE_SIGNATURE.as_ptr() as *const c_char,

    ctx: &ngx_http_awssigv4_module_ctx as *const _ as *mut _,
    commands: unsafe { &ngx_http_awssigv4_commands[0] as *const _ as *mut _ },
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

impl Merge for ModuleConfig {
    fn merge(&mut self, prev: &ModuleConfig) -> Result<(), MergeConfigError> {
        if prev.enable {
            self.enable = true;
        };

        if self.access_key.is_empty() {
            self.access_key = String::from(if !prev.access_key.is_empty() {
                &prev.access_key
            } else {
                ""
            });
        }
        if self.enable && self.access_key.is_empty() {
            return Err(MergeConfigError::NoValue);
        }

        if self.secret_key.is_empty() {
            self.secret_key = String::from(if !prev.secret_key.is_empty() {
                &prev.secret_key
            } else {
                ""
            });
        }
        if self.enable && self.secret_key.is_empty() {
            return Err(MergeConfigError::NoValue);
        }

        if self.s3_bucket.is_empty() {
            self.s3_bucket = String::from(if !prev.s3_bucket.is_empty() {
                &prev.s3_bucket
            } else {
                ""
            });
        }
        if self.enable && self.s3_bucket.is_empty() {
            return Err(MergeConfigError::NoValue);
        }

        if self.s3_endpoint.is_empty() {
            self.s3_endpoint = String::from(if !prev.s3_endpoint.is_empty() {
                &prev.s3_endpoint
            } else {
                "s3.amazonaws.com"
            });
        }
        Ok(())
    }
}

#[no_mangle]
extern "C" fn ngx_http_awssigv4_commands_set_enable(
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

#[no_mangle]
extern "C" fn ngx_http_awssigv4_commands_set_access_key(
    cf: *mut ngx_conf_t,
    _cmd: *mut ngx_command_t,
    conf: *mut c_void,
) -> *mut c_char {
    unsafe {
        let conf = &mut *(conf as *mut ModuleConfig);
        let args = (*(*cf).args).elts as *mut ngx_str_t;
        conf.access_key = (*args.add(1)).to_string();
    };

    std::ptr::null_mut()
}

#[no_mangle]
extern "C" fn ngx_http_awssigv4_commands_set_secret_key(
    cf: *mut ngx_conf_t,
    _cmd: *mut ngx_command_t,
    conf: *mut c_void,
) -> *mut c_char {
    unsafe {
        let conf = &mut *(conf as *mut ModuleConfig);
        let args = (*(*cf).args).elts as *mut ngx_str_t;
        conf.secret_key = (*args.add(1)).to_string();
    };

    std::ptr::null_mut()
}

#[no_mangle]
extern "C" fn ngx_http_awssigv4_commands_set_s3_bucket(
    cf: *mut ngx_conf_t,
    _cmd: *mut ngx_command_t,
    conf: *mut c_void,
) -> *mut c_char {
    unsafe {
        let conf = &mut *(conf as *mut ModuleConfig);
        let args = (*(*cf).args).elts as *mut ngx_str_t;
        conf.s3_bucket = (*args.add(1)).to_string();
        if conf.s3_bucket.len() == 1 {
            println!("Validation failed");
            return ngx::core::NGX_CONF_ERROR as _;
        }
    };
    std::ptr::null_mut()
}

#[no_mangle]
extern "C" fn ngx_http_awssigv4_commands_set_s3_endpoint(
    cf: *mut ngx_conf_t,
    _cmd: *mut ngx_command_t,
    conf: *mut c_void,
) -> *mut c_char {
    unsafe {
        let conf = &mut *(conf as *mut ModuleConfig);
        let args = (*(*cf).args).elts as *mut ngx_str_t;
        conf.s3_endpoint = (*args.add(1)).to_string();
    };

    std::ptr::null_mut()
}

http_request_handler!(awssigv4_header_handler, |request: &mut Request| {
    // get Module Config from request
    let conf = unsafe { request.get_module_loc_conf::<ModuleConfig>(&ngx_http_awssigv4_module) };
    let conf = conf.unwrap();
    ngx_log_debug_http!(request, "AWS signature V4 module {}", {
        if conf.enable {
            "enabled"
        } else {
            "disabled"
        }
    });
    if !conf.enable {
        return core::Status::NGX_DECLINED;
    }

    // TODO: build url properly from the original URL from client
    let method = request.method();
    if !matches!(method, ngx::http::Method::HEAD | ngx::http::Method::GET) {
        return HTTPStatus::FORBIDDEN.into();
    }

    let datetime = chrono::Utc::now();
    let uri = match request.unparsed_uri().to_str() {
        Ok(v) => format!("https://{}.{}{}", conf.s3_bucket, conf.s3_endpoint, v),
        Err(_) => return core::Status::NGX_DECLINED,
    };

    let datetime_now = datetime.format("%Y%m%dT%H%M%SZ");
    let datetime_now = datetime_now.to_string();

    let signature = {
        // NOTE: aws_sign_v4::AwsSign::new() implementation requires a HeaderMap.
        // Iterate over requests headers_in and copy into HeaderMap
        // Copy only headers that will be used to sign the request
        let mut headers = HeaderMap::new();
        for (name, value) in request.headers_in_iterator() {
            match name.to_lowercase().as_str() {
                "host" => {
                    headers.insert(http::header::HOST, value.parse().unwrap());
                }
                &_ => {}
            };
        }
        headers.insert("X-Amz-Date", datetime_now.parse().unwrap());
        ngx_log_debug_http!(request, "headers {:?}", headers);
        ngx_log_debug_http!(request, "method {:?}", method);
        ngx_log_debug_http!(request, "uri {:?}", uri);
        ngx_log_debug_http!(request, "datetime_now {:?}", datetime_now);

        let s = aws_sign_v4::AwsSign::new(
            method.as_str(),
            &uri,
            &datetime,
            &headers,
            "us-east-1",
            conf.access_key.as_str(),
            conf.secret_key.as_str(),
            "s3",
            "",
        );
        s.sign()
    };

    request.add_header_in("authorization", signature.as_str());
    request.add_header_in("X-Amz-Date", datetime_now.as_str());

    // done signing, let's print values we have in request.headers_out, request.headers_in
    for (name, value) in request.headers_out_iterator() {
        ngx_log_debug_http!(request, "headers_out {}: {}", name, value);
    }
    for (name, value) in request.headers_in_iterator() {
        ngx_log_debug_http!(request, "headers_in  {}: {}", name, value);
    }

    core::Status::NGX_OK
});
