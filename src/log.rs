// `fmt::Arguments` in log macros stores references to temporaries and cannot be extracted to a variable,
// thus we are not able to move it out of the unsafe block.
#![allow(clippy::macro_metavars_in_unsafe)]

use std::ffi::CStr;
use std::fmt;

use crate::ffi::{self, ngx_conf_log_error, ngx_err_t, ngx_log_error_core, ngx_log_t, ngx_uint_t};

/// Checks if the message of the specified level should be logged with this logger.
///
/// # Safety
///
/// The function should be called with a valid target pointer.
#[inline]
pub unsafe fn should_log<T: LogTarget>(target: *const T, level: Level) -> bool {
    debug_assert!(!target.is_null());
    let log = (*target).get_log();
    if log.is_null() {
        return false;
    }
    (*log).log_level >= level.into()
}

/// Checks if the debug message with the specified mask should be logged with this logger.
///
/// # Safety
///
/// The function should be called with a valid target pointer.
#[inline]
pub unsafe fn should_debug<T: LogTarget>(target: *const T, mask: Option<DebugMask>) -> bool {
    debug_assert!(!target.is_null());
    let log = (*target).get_log();
    if log.is_null() {
        return false;
    }
    let mask: u32 = mask.unwrap_or((*target).debug_mask()).into();
    (*log).log_level & mask as usize != 0
}

/// Writes [std::fmt::Arguments] into the nginx logger.
///
/// # Safety
///
/// The function should be called with a valid target pointer.
#[inline]
pub unsafe fn log_error<T: LogTarget>(target: *const T, level: Level, err: ngx_err_t, args: fmt::Arguments<'_>) {
    debug_assert!(!target.is_null());
    if let Some(str) = args.as_str() {
        (*target).write_log(level, err, str.as_bytes());
    } else {
        (*target).write_log(level, err, args.to_string().as_bytes());
    }
}

/// Severity level
#[repr(usize)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Level {
    /// System is unusable
    Emerg = ffi::NGX_LOG_EMERG as usize,
    /// Action must be taken immediately
    Alert = ffi::NGX_LOG_ALERT as usize,
    /// Critical conditions
    Crit = ffi::NGX_LOG_CRIT as usize,
    /// Error conditions
    Err = ffi::NGX_LOG_ERR as usize,
    /// Warning conditions
    Warn = ffi::NGX_LOG_WARN as usize,
    /// Normal but significant condition
    Notice = ffi::NGX_LOG_NOTICE as usize,
    /// Informational messages
    Info = ffi::NGX_LOG_INFO as usize,
    /// Debug-level messages
    Debug = ffi::NGX_LOG_DEBUG as usize,
}

impl From<Level> for ngx_uint_t {
    #[inline]
    fn from(value: Level) -> Self {
        value as ngx_uint_t
    }
}

/// Utility trait for nginx structures that contain logger objects
pub trait LogTarget {
    /// Default debug mask for this target
    #[inline]
    fn debug_mask(&self) -> DebugMask {
        DebugMask::Core
    }

    /// Returns `ngx_log_t` owned by the target
    fn get_log(&self) -> *const ngx_log_t;

    /// Low-level implementation for writing byte slice into the nginx logger
    #[inline]
    fn write_log(&self, level: Level, err: ngx_err_t, message: &[u8]) {
        const FORMAT: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"%*s\0") };
        let log = self.get_log().cast_mut();
        unsafe { ngx_log_error_core(level.into(), log, err, FORMAT.as_ptr(), message.len(), message.as_ptr()) };
    }
}

/// Implementations for the main types
impl LogTarget for ngx_log_t {
    #[inline]
    fn get_log(&self) -> *const ngx_log_t {
        self
    }
}

impl LogTarget for ffi::ngx_cycle_t {
    #[inline]
    fn get_log(&self) -> *const ngx_log_t {
        self.log
    }
}

impl LogTarget for ffi::ngx_conf_t {
    #[inline]
    fn get_log(&self) -> *const ngx_log_t {
        self.log
    }

    #[inline]
    fn write_log(&self, level: Level, err: ngx_err_t, msg: &[u8]) {
        const FORMAT: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"%*s\0") };
        if level < Level::Debug {
            // ngx_conf_log_error will not mutate the cf argument.
            // ngx_log_error_core may mutate the log argument, but cf does not own the log.
            let cf = self as *const _ as *mut _;
            unsafe { ngx_conf_log_error(level.into(), cf, err, FORMAT.as_ptr(), msg.len(), msg.as_ptr()) };
        } else {
            // Debug messages don't need the configuration file context
            // SAFETY: this should called after `should_log` or `should_debug`, when we already know
            // that the log pointer is valid
            unsafe { &*self.log }.write_log(level, err, msg);
        }
    }
}

impl LogTarget for ffi::ngx_event_t {
    #[inline]
    fn debug_mask(&self) -> DebugMask {
        DebugMask::Event
    }

    #[inline]
    fn get_log(&self) -> *const ngx_log_t {
        self.log
    }
}

/// Write to logger at a specified [Level].
#[macro_export]
macro_rules! ngx_log_error {
    ( $level:expr, $log:expr, $($arg:tt)* ) => {
        if unsafe { $crate::log::should_log($log, $level) } {
            unsafe { $crate::log::log_error($log, $level, 0, format_args!($($arg)*)) };
        }
    }
}

/// Write to logger at debug level.
#[macro_export]
macro_rules! ngx_log_debug {
    ( $log:expr, $($arg:tt)* ) => {
        if unsafe { $crate::log::should_debug($log, None) } {
            unsafe { $crate::log::log_error($log, $crate::log::Level::Debug, 0, format_args!($($arg)*)) };
        }
    }
}

/// Log to request connection log at level [`NGX_LOG_DEBUG_HTTP`].
///
/// [`NGX_LOG_DEBUG_HTTP`]: https://nginx.org/en/docs/dev/development_guide.html#logging
#[macro_export]
macro_rules! ngx_log_debug_http {
    ( $request:expr, $($arg:tt)* ) => {
        if unsafe { $crate::log::should_debug($request, None) } {
            unsafe { $crate::log::log_error($request, $crate::log::Level::Debug, 0, format_args!($($arg)*)) };
        }
    }
}

/// Log with requested debug mask.
///
/// **NOTE:** This macro supports [`DebugMask::Http`] (`NGX_LOG_DEBUG_HTTP`), however, if you have
/// access to a Request object via an http handler it can be more convenient and readable to use
/// the [`ngx_log_debug_http`] macro instead.
///
/// See <https://nginx.org/en/docs/dev/development_guide.html#logging> for details and available
/// masks.
#[macro_export]
macro_rules! ngx_log_debug_mask {
    ( DebugMask::Core, $log:expr, $($arg:tt)* ) => ({
        if unsafe { $crate::log::should_debug($log, Some(DebugMask::Core)) } {
            unsafe { $crate::log::log_error($log, $crate::log::Level::Debug, 0, format_args!($($arg)*)) };
        }
    });
    ( DebugMask::Alloc, $log:expr, $($arg:tt)* ) => ({
        if unsafe { $crate::log::should_debug($log, Some(DebugMask::Alloc)) } {
            unsafe { $crate::log::log_error($log, $crate::log::Level::Debug, 0, format_args!($($arg)*)) };
        }
    });
    ( DebugMask::Mutex, $log:expr, $($arg:tt)* ) => ({
        if unsafe { $crate::log::should_debug($log, Some(DebugMask::Mutex)) } {
            unsafe { $crate::log::log_error($log, $crate::log::Level::Debug, 0, format_args!($($arg)*)) };
        }
    });
    ( DebugMask::Event, $log:expr, $($arg:tt)* ) => ({
        if unsafe { $crate::log::should_debug($log, Some(DebugMask::Event)) } {
            unsafe { $crate::log::log_error($log, $crate::log::Level::Debug, 0, format_args!($($arg)*)) };
        }
    });
    ( DebugMask::Http, $log:expr, $($arg:tt)* ) => ({
        if unsafe { $crate::log::should_debug($log, Some(DebugMask::Http))} {
            unsafe { $crate::log::log_error($log, $crate::log::Level::Debug, 0, format_args!($($arg)*)) };
        }
    });
    ( DebugMask::Mail, $log:expr, $($arg:tt)* ) => ({
        if unsafe { $crate::log::should_debug($log, Some(DebugMask::Mail)) } {
            unsafe { $crate::log::log_error($log, $crate::log::Level::Debug, 0, format_args!($($arg)*)) };
        }
    });
    ( DebugMask::Stream, $log:expr, $($arg:tt)* ) => ({
        if unsafe { $crate::log::should_debug($log, Some(DebugMask::Stream)) } {
            unsafe { $crate::log::log_error($log, $crate::log::Level::Debug, 0, format_args!($($arg)*)) };
        }
    });
}

/// Debug masks for use with [`ngx_log_debug_mask`], these represent the only accepted values for
/// the mask.
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

#[cfg(test)]
mod tests {

    use super::*;

    struct MockLog {
        log: ngx_log_t,
        buffer: std::cell::Cell<Vec<u8>>,
    }

    impl MockLog {
        fn new(level: u32) -> Self {
            let mut inst = MockLog {
                log: unsafe { std::mem::zeroed() },
                buffer: vec![].into(),
            };
            inst.log.log_level = level as _;
            inst
        }
    }

    impl LogTarget for MockLog {
        fn get_log(&self) -> *const ngx_log_t {
            &self.log
        }

        fn write_log(&self, _level: Level, _err: ngx_err_t, message: &[u8]) {
            self.buffer.set(message.to_vec());
        }
    }

    #[test]
    fn test_mask_lower_bound() {
        assert!(<DebugMask as Into<u32>>::into(DebugMask::Core) == crate::ffi::NGX_LOG_DEBUG_FIRST);
    }
    #[test]
    fn test_mask_upper_bound() {
        assert!(<DebugMask as Into<u32>>::into(DebugMask::Stream) == crate::ffi::NGX_LOG_DEBUG_LAST);
    }
    #[test]
    fn test_mask() {
        let log = MockLog::new(crate::ffi::NGX_LOG_DEBUG_CORE);

        let mut r = unsafe { should_debug(&log, None) };
        assert!(r);

        r = unsafe { should_debug(&log, Some(DebugMask::Core)) };
        assert!(r);

        r = unsafe { should_debug(&log, Some(DebugMask::Alloc)) };
        assert!(!r);

        ngx_log_debug!(&log, "mask-core-default");
        assert_eq!(log.buffer.take(), b"mask-core-default");

        ngx_log_debug_mask!(DebugMask::Core, &log, "mask-core");
        assert_eq!(log.buffer.take(), b"mask-core");

        ngx_log_debug_mask!(DebugMask::Alloc, &log, "mask-alloc");
        assert_ne!(log.buffer.take(), b"mask-alloc");
    }
    #[test]
    fn test_level() {
        let log = MockLog::new(crate::ffi::NGX_LOG_NOTICE);

        ngx_log_error!(Level::Warn, &log, "level-warn");
        assert_eq!(log.buffer.take(), b"level-warn");

        ngx_log_error!(Level::Notice, &log, "level-notice");
        assert_eq!(log.buffer.take(), b"level-notice");

        ngx_log_error!(Level::Info, &log, "level-info");
        assert_ne!(log.buffer.take(), b"level-info");

        ngx_log_error!(Level::Debug, &log, "level-debug");
        assert_ne!(log.buffer.take(), b"level-debug");

        ngx_log_error!(Level::Err, &log, "level-err");
        assert_eq!(log.buffer.take(), b"level-err");
    }
}
