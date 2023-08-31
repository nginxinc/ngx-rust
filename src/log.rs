/// Utility function to provide typed checking of the mask's field state.
#[inline(always)]
pub fn check_mask(mask: DebugMask, log_level: usize) -> bool {
    let mask_bits: u32 = mask.into();
    if log_level & mask_bits as usize == 0 {
        return false;
    }
    true
}

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

/// Debug masks for use with ngx_log_debug_mask, these represent the only accepted values for the
/// mask.
#[derive(Debug)]
pub enum DebugMask {
    /// Aligns to the NGX_LOG_DEBUG_CORE mask.
    Core,
    /// Aligns to the NGX_LOG_DEBUG_ALLOC mask.
    Alloc,
    /// Aligns to the NGX_LOG_DEBUG_MUTEX mask.
    Mutex,
    /// Aligns to the NGX_LOG_DEBUG_EVENT mask.
    Event,
    /// Aligns to the NGX_LOG_DEBUG_HTTP mask.
    Http,
    /// Aligns to the NGX_LOG_DEBUG_MAIL mask.
    Mail,
    /// Aligns to the NGX_LOG_DEBUG_STREAM mask.
    Stream,
}

impl TryFrom<u32> for DebugMask {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            crate::ffi::NGX_LOG_DEBUG_CORE => Ok(DebugMask::Core),
            crate::ffi::NGX_LOG_DEBUG_ALLOC => Ok(DebugMask::Alloc),
            crate::ffi::NGX_LOG_DEBUG_MUTEX => Ok(DebugMask::Mutex),
            crate::ffi::NGX_LOG_DEBUG_EVENT => Ok(DebugMask::Event),
            crate::ffi::NGX_LOG_DEBUG_HTTP => Ok(DebugMask::Http),
            crate::ffi::NGX_LOG_DEBUG_MAIL => Ok(DebugMask::Mail),
            crate::ffi::NGX_LOG_DEBUG_STREAM => Ok(DebugMask::Stream),
            _ => Err(0),
        }
    }
}

impl From<DebugMask> for u32 {
    fn from(value: DebugMask) -> Self {
        match value {
            DebugMask::Core => crate::ffi::NGX_LOG_DEBUG_CORE,
            DebugMask::Alloc => crate::ffi::NGX_LOG_DEBUG_ALLOC,
            DebugMask::Mutex => crate::ffi::NGX_LOG_DEBUG_MUTEX,
            DebugMask::Event => crate::ffi::NGX_LOG_DEBUG_EVENT,
            DebugMask::Http => crate::ffi::NGX_LOG_DEBUG_HTTP,
            DebugMask::Mail => crate::ffi::NGX_LOG_DEBUG_MAIL,
            DebugMask::Stream => crate::ffi::NGX_LOG_DEBUG_STREAM,
        }
    }
}

/// Log with requested debug mask.
///
/// **NOTE:** This macro supports `DebugMask::Http` (`NGX_LOG_DEBUG_HTTP`), however, if you have
/// access to a Request object via an http handler it can be more convenient and readable to use the
/// `ngx_log_debug_http` macro instead.
///
/// See https://nginx.org/en/docs/dev/development_guide.html#logging for details and available
/// masks.
#[macro_export]
macro_rules! ngx_log_debug_mask {
    ( DebugMask::Core, $log:expr, $($arg:tt)* ) => ({
        let log_level = unsafe { (*$log).log_level };
        if $crate::log::check_mask(DebugMask::Core, log_level) {
            let level = $crate::ffi::NGX_LOG_DEBUG as $crate::ffi::ngx_uint_t;
            let fmt = ::std::ffi::CString::new("%s").unwrap();
            let c_message = ::std::ffi::CString::new(format!($($arg)*)).unwrap();
            unsafe {
                $crate::ffi::ngx_log_error_core(level, $log, 0, fmt.as_ptr(), c_message.as_ptr());
            }
        }
    });
    ( DebugMask::Alloc, $log:expr, $($arg:tt)* ) => ({
        let log_level = unsafe { (*$log).log_level };
        if $crate::log::check_mask(DebugMask::Alloc, log_level) {
            let level = $crate::ffi::NGX_LOG_DEBUG as $crate::ffi::ngx_uint_t;
            let fmt = ::std::ffi::CString::new("%s").unwrap();
            let c_message = ::std::ffi::CString::new(format!($($arg)*)).unwrap();
            unsafe {
                $crate::ffi::ngx_log_error_core(level, $log, 0, fmt.as_ptr(), c_message.as_ptr());
            }
        }
    });
    ( DebugMask::Mutex, $log:expr, $($arg:tt)* ) => ({
        let log_level = unsafe { (*$log).log_level };
        if $crate::log::check_mask(DebugMask::Mutex, log_level) {
            let level = $crate::ffi::NGX_LOG_DEBUG as $crate::ffi::ngx_uint_t;
            let fmt = ::std::ffi::CString::new("%s").unwrap();
            let c_message = ::std::ffi::CString::new(format!($($arg)*)).unwrap();
            unsafe {
                $crate::ffi::ngx_log_error_core(level, $log, 0, fmt.as_ptr(), c_message.as_ptr());
            }
        }
    });
    ( DebugMask::Event, $log:expr, $($arg:tt)* ) => ({
        let log_level = unsafe { (*$log).log_level };
        if $crate::log::check_mask(DebugMask::Event, log_level) {
            let level = $crate::ffi::NGX_LOG_DEBUG as $crate::ffi::ngx_uint_t;
            let fmt = ::std::ffi::CString::new("%s").unwrap();
            let c_message = ::std::ffi::CString::new(format!($($arg)*)).unwrap();
            unsafe {
                $crate::ffi::ngx_log_error_core(level, $log, 0, fmt.as_ptr(), c_message.as_ptr());
            }
        }
    });
    ( DebugMask::Http, $log:expr, $($arg:tt)* ) => ({
        let log_level = unsafe { (*$log).log_level };
        if $crate::log::check_mask(DebugMask::Http, log_level) {
            let level = $crate::ffi::NGX_LOG_DEBUG as $crate::ffi::ngx_uint_t;
            let fmt = ::std::ffi::CString::new("%s").unwrap();
            let c_message = ::std::ffi::CString::new(format!($($arg)*)).unwrap();
            unsafe {
                $crate::ffi::ngx_log_error_core(level, $log, 0, fmt.as_ptr(), c_message.as_ptr());
            }
        }
    });
    ( DebugMask::Mail, $log:expr, $($arg:tt)* ) => ({
        let log_level = unsafe { (*$log).log_level };
        if $crate::log::check_mask(DebugMask::Mail, log_level) {
            let level = $crate::ffi::NGX_LOG_DEBUG as $crate::ffi::ngx_uint_t;
            let fmt = ::std::ffi::CString::new("%s").unwrap();
            let c_message = ::std::ffi::CString::new(format!($($arg)*)).unwrap();
            unsafe {
                $crate::ffi::ngx_log_error_core(level, $log, 0, fmt.as_ptr(), c_message.as_ptr());
            }
        }
    });
    ( DebugMask::Stream, $log:expr, $($arg:tt)* ) => ({
        let log_level = unsafe { (*$log).log_level };
        if $crate::log::check_mask(DebugMask::Stream, log_level) {
            let level = $crate::ffi::NGX_LOG_DEBUG as $crate::ffi::ngx_uint_t;
            let fmt = ::std::ffi::CString::new("%s").unwrap();
            let c_message = ::std::ffi::CString::new(format!($($arg)*)).unwrap();
            unsafe {
                $crate::ffi::ngx_log_error_core(level, $log, 0, fmt.as_ptr(), c_message.as_ptr());
            }
        }
    });
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_mask_lower_bound() {
        assert!(<DebugMask as Into<u32>>::into(DebugMask::Core) == crate::ffi::NGX_LOG_DEBUG_FIRST);
    }
    #[test]
    fn test_mask_upper_bound() {
        assert!(<DebugMask as Into<u32>>::into(DebugMask::Stream) == crate::ffi::NGX_LOG_DEBUG_LAST);
    }
    #[test]
    fn test_check_mask() {
        struct MockLog {
            log_level: usize,
        }
        let mock = MockLog { log_level: 16 };

        let mut r = check_mask(DebugMask::Core, mock.log_level);
        assert!(r);

        r = check_mask(DebugMask::Alloc, mock.log_level);
        assert!(!r);
    }
}
