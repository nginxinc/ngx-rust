// Utility function to provide typed checking of the mask's field state.
#[inline(always)]
fn check_mask(mask: DebugMasks, log_level: usize) -> bool {
    let mask_bits: u32 = mask.into();
    if log_level & mask_bits as usize == 0 {
        return false;
    }
    true
}

// Internal macro, provided to reduce code duplication.
//
// Expects an ngx_log_t and message format template.
macro_rules! _ngx_log_debug_internal {
    ( $log:expr, $($arg:tt)* ) => {
        let level = $crate::ffi::NGX_LOG_DEBUG as $crate::ffi::ngx_uint_t;
        let fmt = ::std::ffi::CString::new("%s").unwrap();
        let c_message = ::std::ffi::CString::new(format!($($arg)*)).unwrap();
        unsafe {
            $crate::ffi::ngx_log_error_core(level, $log, 0, fmt.as_ptr(), c_message.as_ptr());
        }
    }
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
            $crate::_ngx_log_debug_internal!($log, $($arg)*);
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
pub enum DebugMasks {
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

impl TryFrom<u32> for DebugMasks {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            crate::ffi::NGX_LOG_DEBUG_CORE => Ok(DebugMasks::Core),
            crate::ffi::NGX_LOG_DEBUG_ALLOC => Ok(DebugMasks::Alloc),
            crate::ffi::NGX_LOG_DEBUG_MUTEX => Ok(DebugMasks::Mutex),
            crate::ffi::NGX_LOG_DEBUG_EVENT => Ok(DebugMasks::Event),
            crate::ffi::NGX_LOG_DEBUG_HTTP => Ok(DebugMasks::Http),
            crate::ffi::NGX_LOG_DEBUG_MAIL => Ok(DebugMasks::Mail),
            crate::ffi::NGX_LOG_DEBUG_STREAM => Ok(DebugMasks::Stream),
            _ => Err(0),
        }
    }
}

impl From<DebugMasks> for u32 {
    fn from(value: DebugMasks) -> Self {
        match value {
            DebugMasks::Core => crate::ffi::NGX_LOG_DEBUG_CORE,
            DebugMasks::Alloc => crate::ffi::NGX_LOG_DEBUG_ALLOC,
            DebugMasks::Mutex => crate::ffi::NGX_LOG_DEBUG_MUTEX,
            DebugMasks::Event => crate::ffi::NGX_LOG_DEBUG_EVENT,
            DebugMasks::Http => crate::ffi::NGX_LOG_DEBUG_HTTP,
            DebugMasks::Mail => crate::ffi::NGX_LOG_DEBUG_MAIL,
            DebugMasks::Stream => crate::ffi::NGX_LOG_DEBUG_STREAM,
        }
    }
}

/// Log with appropriate debug mask.
///
/// When the request logger is available `ngx_log_debug_http` can be used for `NGX_LOG_DEBUG_HTTP` masks.
/// This macro is useful when other masks are necessary or when the request logger is not
/// conveniently accessible.
///
/// See https://nginx.org/en/docs/dev/development_guide.html#logging for details and available
/// masks.
#[macro_export]
macro_rules! ngx_log_debug_mask {
    ( DebugMasks::Core, $log:expr, $($arg:tt)* ) => ({
        let log_level = unsafe { (*$log).log_level };
        if check_mask(DebugMasks::Core, log_level) {
            $crate::_ngx_log_debug_internal!(log, $($arg:tt)*);
        }
    });
    ( DebugMasks::Alloc, $log:expr, $($arg:tt)* ) => ({
        let log_level = unsafe { (*$log).log_level };
        if check_mask(DebugMasks::Alloc, log_level) {
            $crate::_ngx_log_debug_internal!(log, $($arg:tt)*);
        }
    });
    ( DebugMasks::Mutex, $log:expr, $($arg:tt)* ) => ({
        let log_level = unsafe { (*$log).log_level };
        if check_mask(DebugMasks::Mutex, log_level) {
            $crate::_ngx_log_debug_internal!(log, $($arg:tt)*);
        }
    });
    ( DebugMasks::Event, $log:expr, $($arg:tt)* ) => ({
        let log_level = unsafe { (*$log).log_level };
        if check_mask(DebugMasks::Event, log_level) {
            $crate::_ngx_log_debug_internal!(log, $($arg:tt)*);
        }
    });
    ( DebugMasks::Http, $log:expr, $($arg:tt)* ) => ({
        let log_level = unsafe { (*$log).log_level };
        if check_mask(DebugMasks::Http, log_level) {
            $crate::_ngx_log_debug_internal!(log, $($arg:tt)*);
        }
    });
    ( DebugMasks::Mail, $log:expr, $($arg:tt)* ) => ({
        let log_level = unsafe { (*$log).log_level };
        if check_mask(DebugMasks::Mail, log_level) {
            $crate::_ngx_log_debug_internal!(log, $($arg:tt)*);
        }
    });
    ( DebugMasks::Stream, $log:expr, $($arg:tt)* ) => ({
        let log_level = unsafe { (*$log).log_level };
        if check_mask(DebugMasks::Stream, log_level) {
            $crate::_ngx_log_debug_internal!(log, $($arg:tt)*);
        }
    });
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_mask_lower_bound() {
        assert!(<DebugMasks as Into<u32>>::into(DebugMasks::Core) == crate::ffi::NGX_LOG_DEBUG_FIRST);
    }
    #[test]
    fn test_mask_upper_bound() {
        assert!(<DebugMasks as Into<u32>>::into(DebugMasks::Stream) == crate::ffi::NGX_LOG_DEBUG_LAST);
    }
    #[test]
    fn test_check_mask() {
        struct MockLog {
            log_level: usize,
        }
        let mock = MockLog { log_level: 16 };

        let mut r = check_mask(DebugMasks::Core, mock.log_level);
        assert!(r == true);

        r = check_mask(DebugMasks::Alloc, mock.log_level);
        assert!(r == false);
    }
}
