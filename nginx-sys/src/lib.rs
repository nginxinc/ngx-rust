//! # nginx-sys
//!
//! The `nginx-sys` crate provides low-level bindings for the nginx C API, allowing Rust applications to interact with nginx servers and modules.
//!
//! ## Usage
//!
//! Add `nginx-sys` as a dependency in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! nginx-sys = "0.1.0"
//! ```
//!
//! ## Features
//!
//! - `build`: Enables the build scripts to compile and link against the nginx C library. This feature is enabled by default.
//!
//! ## Examples
//!
//! ### Get Nginx Version
//!
//! This example demonstrates how to retrieve the version of the nginx server.
//!
//! ```rust,no_run
//! use nginx_sys::nginx_version;
//!
//! let version = unsafe { nginx_version() };
//! println!("Nginx version: {}", version);
//! ```
//!
#![warn(missing_docs)]

use std::fmt;
use std::ptr::copy_nonoverlapping;
use std::slice;

#[doc(hidden)]
mod bindings {
    #![allow(missing_docs)]
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(clippy::all)]
    #![allow(improper_ctypes)]
    #![allow(rustdoc::broken_intra_doc_links)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
#[doc(no_inline)]
pub use bindings::*;

/// Convert a string slice (`&str`) to a raw pointer (`*mut u_char`) allocated in the given nginx memory pool.
///
/// # Arguments
///
/// * `pool` - A pointer to the nginx memory pool (`ngx_pool_t`).
/// * `data` - The string slice to convert to a raw pointer.
///
/// # Safety
/// This function is marked as unsafe because it involves raw pointer manipulation and direct memory allocation using `ngx_palloc`.
///
/// # Returns
/// A raw pointer (`*mut u_char`) to the allocated memory containing the converted string data.
///
/// # Example
/// ```rust
/// let pool: *mut ngx_pool_t = ...; // Obtain a pointer to the nginx memory pool
/// let data: &str = "example"; // The string to convert
/// let ptr = str_to_uchar(pool, data);
/// ```
pub unsafe fn str_to_uchar(pool: *mut ngx_pool_t, data: &str) -> *mut u_char {
    let ptr: *mut u_char = ngx_palloc(pool, data.len() as _) as _;
    copy_nonoverlapping(data.as_ptr(), ptr, data.len());
    ptr
}

impl ngx_str_t {
    /// Convert the nginx string to a string slice (`&str`).
    ///
    /// # Safety
    /// This function is marked as unsafe because it involves raw pointer manipulation.
    /// It assumes that the underlying `data` pointer is valid and points to a valid UTF-8 encoded string.
    ///
    /// # Returns
    /// A string slice (`&str`) representing the nginx string.
    pub fn to_str(&self) -> &str {
        unsafe {
            let slice = slice::from_raw_parts(self.data, self.len);
            return std::str::from_utf8(slice).unwrap();
        }
    }

    /// Create an `ngx_str_t` instance from a `String`.
    ///
    /// # Arguments
    ///
    /// * `pool` - A pointer to the nginx memory pool (`ngx_pool_t`).
    /// * `data` - The `String` from which to create the nginx string.
    ///
    /// # Safety
    /// This function is marked as unsafe because it accepts a raw pointer argument. There is no
    /// way to know if `pool` is pointing to valid memory. The caller must provide a valid pool to
    /// avoid indeterminate behavior.
    ///
    /// # Returns
    /// An `ngx_str_t` instance representing the given `String`.
    pub unsafe fn from_string(pool: *mut ngx_pool_t, data: String) -> Self {
        ngx_str_t {
            data: str_to_uchar(pool, data.as_str()),
            len: data.len() as _,
        }
    }

    /// Create an `ngx_str_t` instance from a string slice (`&str`).
    ///
    /// # Arguments
    ///
    /// * `pool` - A pointer to the nginx memory pool (`ngx_pool_t`).
    /// * `data` - The string slice from which to create the nginx string.
    ///
    /// # Safety
    /// This function is marked as unsafe because it accepts a raw pointer argument. There is no
    /// way to know if `pool` is pointing to valid memory. The caller must provide a valid pool to
    /// avoid indeterminate behavior.
    ///
    /// # Returns
    /// An `ngx_str_t` instance representing the given string slice.
    pub unsafe fn from_str(pool: *mut ngx_pool_t, data: &str) -> Self {
        ngx_str_t {
            data: str_to_uchar(pool, data),
            len: data.len() as _,
        }
    }
}

impl From<ngx_str_t> for &[u8] {
    fn from(s: ngx_str_t) -> Self {
        if s.len == 0 || s.data.is_null() {
            return Default::default();
        }
        unsafe { slice::from_raw_parts(s.data, s.len) }
    }
}

impl TryFrom<ngx_str_t> for String {
    type Error = std::string::FromUtf8Error;

    fn try_from(s: ngx_str_t) -> Result<Self, Self::Error> {
        let bytes: &[u8] = s.into();
        String::from_utf8(bytes.into())
    }
}

impl fmt::Display for ngx_str_t {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy((*self).into()))
    }
}

impl TryFrom<ngx_str_t> for &str {
    type Error = std::str::Utf8Error;

    fn try_from(s: ngx_str_t) -> Result<Self, Self::Error> {
        std::str::from_utf8(s.into())
    }
}

/// Add a key-value pair to an nginx table entry (`ngx_table_elt_t`) in the given nginx memory pool.
///
/// # Arguments
///
/// * `table` - A pointer to the nginx table entry (`ngx_table_elt_t`) to modify.
/// * `pool` - A pointer to the nginx memory pool (`ngx_pool_t`) for memory allocation.
/// * `key` - The key string to add to the table entry.
/// * `value` - The value string to add to the table entry.
///
/// # Safety
/// This function is marked as unsafe because it involves raw pointer manipulation and direct memory allocation using `str_to_uchar`.
///
/// # Returns
/// An `Option<()>` representing the result of the operation. `Some(())` indicates success, while `None` indicates a null table pointer.
///
/// # Example
/// ```rust
/// let table: *mut ngx_table_elt_t = ...; // Obtain a pointer to the nginx table entry
/// let pool: *mut ngx_pool_t = ...; // Obtain a pointer to the nginx memory pool
/// let key: &str = "key"; // The key to add
/// let value: &str = "value"; // The value to add
/// let result = add_to_ngx_table(table, pool, key, value);
/// ```
pub unsafe fn add_to_ngx_table(
    table: *mut ngx_table_elt_t,
    pool: *mut ngx_pool_t,
    key: &str,
    value: &str,
) -> Option<()> {
    if table.is_null() {
        return None;
    }
    table.as_mut().map(|table| {
        table.hash = 1;
        table.key.len = key.len() as _;
        table.key.data = str_to_uchar(pool, key);
        table.value.len = value.len() as _;
        table.value.data = str_to_uchar(pool, value);
        table.lowcase_key = str_to_uchar(pool, String::from(key).to_ascii_lowercase().as_str());
    })
}
