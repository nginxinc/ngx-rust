/// Write to logger at a specified level.
///
/// See [Logging](https://nginx.org/en/docs/dev/development_guide.html#logging)
/// for available log levels.
#[macro_export]
macro_rules! ngx_log_debug {
    ( $log:expr, $($arg:tt)* ) => {
        let log_level = unsafe { (*$log).log_level };
        if log_level != 0 {
            let level = $crate::ffi::NGX_LOG_DEBUG as $crate::ffi::ngx_uint_t;
            let fmt = ::std::ffi::CString::new("%s").unwrap();
            let c_message = ::std::ffi::CString::new(format!($($arg)*)).unwrap();
            unsafe {
                $crate::ffi::ngx_log_error_core(level, $log, 0, fmt.as_ptr(), c_message.as_ptr());
            }
        }
    }
}

/// Log to request connection log at level [`NGX_LOG_DEBUG_HTTP`].
///
/// [`NGX_LOG_DEBUG_HTTP`]: https://nginx.org/en/docs/dev/development_guide.html#logging
#[macro_export]
macro_rules! ngx_log_debug_http {
    ( $request:expr, $($arg:tt)* ) => {
        let log = unsafe { (*$request.connection()).log };
        $crate::ngx_log_debug!(log, $($arg)*);
    }
}

#[macro_export]
macro_rules! ngx_log_debug_mask {
    ( $mask:expr, $log:expr, $($arg:tt)* ) => {
        let log_level = unsafe { (*$log).log_level };
        if log_level & $mask as usize != 0 {
            let level = $mask as $crate::ffi::ngx_uint_t;
            let fmt = ::std::ffi::CString::new("%s").unwrap();
            let c_message = ::std::ffi::CString::new(format!($($arg)*)).unwrap();
            unsafe {
                $crate::ffi::ngx_log_error_core(level, $log, 0, fmt.as_ptr(), c_message.as_ptr());
            }
        }
    }
}
