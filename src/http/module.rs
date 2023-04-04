use crate::core::NGX_CONF_ERROR;
use crate::core::*;
use crate::ffi::*;

use core::ptr;
use std::os::raw::{c_char, c_void};

#[derive(Debug)]
pub enum MergeConfigError {
    /// No value provided for configuration argument
    NoValue,
}

impl std::error::Error for MergeConfigError {}

impl std::fmt::Display for MergeConfigError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MergeConfigError::NoValue => "no value".fmt(fmt),
        }
    }
}

pub trait Merge {
    fn merge(&mut self, prev: &Self) -> Result<(), MergeConfigError>;
}

impl Merge for () {
    fn merge(&mut self, _prev: &Self) -> Result<(), MergeConfigError> {
        Ok(())
    }
}

pub trait HTTPModule {
    type MainConf: Merge + Default;
    type SrvConf: Merge + Default;
    type LocConf: Merge + Default;

    /// # Safety
    ///
    /// Callers should provide valid non-null `ngx_conf_t` arguments. Implementers must
    /// guard against null inputs or risk runtime errors.
    unsafe extern "C" fn preconfiguration(_cf: *mut ngx_conf_t) -> ngx_int_t {
        Status::NGX_OK.into()
    }

    /// # Safety
    ///
    /// Callers should provide valid non-null `ngx_conf_t` arguments. Implementers must
    /// guard against null inputs or risk runtime errors.
    unsafe extern "C" fn postconfiguration(_cf: *mut ngx_conf_t) -> ngx_int_t {
        Status::NGX_OK.into()
    }

    /// # Safety
    ///
    /// Callers should provide valid non-null `ngx_conf_t` arguments. Implementers must
    /// guard against null inputs or risk runtime errors.
    unsafe extern "C" fn create_main_conf(cf: *mut ngx_conf_t) -> *mut c_void {
        let mut pool = Pool::from_ngx_pool((*cf).pool);
        pool.allocate::<Self::MainConf>(Default::default()) as *mut c_void
    }

    /// # Safety
    ///
    /// Callers should provide valid non-null `ngx_conf_t` arguments. Implementers must
    /// guard against null inputs or risk runtime errors.
    unsafe extern "C" fn init_main_conf(_cf: *mut ngx_conf_t, _conf: *mut c_void) -> *mut c_char {
        ptr::null_mut()
    }

    /// # Safety
    ///
    /// Callers should provide valid non-null `ngx_conf_t` arguments. Implementers must
    /// guard against null inputs or risk runtime errors.
    unsafe extern "C" fn create_srv_conf(cf: *mut ngx_conf_t) -> *mut c_void {
        let mut pool = Pool::from_ngx_pool((*cf).pool);
        pool.allocate::<Self::SrvConf>(Default::default()) as *mut c_void
    }

    /// # Safety
    ///
    /// Callers should provide valid non-null `ngx_conf_t` arguments. Implementers must
    /// guard against null inputs or risk runtime errors.
    unsafe extern "C" fn merge_srv_conf(_cf: *mut ngx_conf_t, prev: *mut c_void, conf: *mut c_void) -> *mut c_char {
        let prev = &mut *(prev as *mut Self::SrvConf);
        let conf = &mut *(conf as *mut Self::SrvConf);
        match conf.merge(prev) {
            Ok(_) => ptr::null_mut(),
            Err(_) => NGX_CONF_ERROR as _,
        }
    }

    /// # Safety
    ///
    /// Callers should provide valid non-null `ngx_conf_t` arguments. Implementers must
    /// guard against null inputs or risk runtime errors.
    unsafe extern "C" fn create_loc_conf(cf: *mut ngx_conf_t) -> *mut c_void {
        let mut pool = Pool::from_ngx_pool((*cf).pool);
        pool.allocate::<Self::LocConf>(Default::default()) as *mut c_void
    }

    /// # Safety
    ///
    /// Callers should provide valid non-null `ngx_conf_t` arguments. Implementers must
    /// guard against null inputs or risk runtime errors.
    unsafe extern "C" fn merge_loc_conf(_cf: *mut ngx_conf_t, prev: *mut c_void, conf: *mut c_void) -> *mut c_char {
        let prev = &mut *(prev as *mut Self::LocConf);
        let conf = &mut *(conf as *mut Self::LocConf);
        match conf.merge(prev) {
            Ok(_) => ptr::null_mut(),
            Err(_) => NGX_CONF_ERROR as _,
        }
    }
}
