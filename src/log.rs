
#[macro_export]
macro_rules! ngx_debug  {

    ($level:expr,$log:expr,$($arg:tt)*) => {
        if (*$log).log_level & $level as usize > 0{
            let c_message = ::std::ffi::CString::new(format!($($arg)*)).unwrap();
            $crate::bindings::ngx_log_error_core($crate::bindings::NGX_LOG_DEBUG as usize, $log, 0, c_message.as_ptr());
        }
    }
}

#[macro_export]
macro_rules! ngx_http_debug  {

    ($request:expr,$($arg:tt)*) => {
        unsafe  {
            ngx_debug!($crate::bindings::NGX_LOG_DEBUG_HTTP,(*($request).connection).log,$($arg)*);
        }
    }
}


#[macro_export]
macro_rules! ngx_event_debug  {

    ($($arg:tt)*) => {
        unsafe  {
            ngx_debug!($crate::bindings::NGX_LOG_DEBUG_EVENT,(*$crate::bindings::ngx_cycle).log,$($arg)*);
        }
    }
}





