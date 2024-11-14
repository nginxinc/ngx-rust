#![doc = include_str!("../README.md")]
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

/// Convert a byte slice to a raw pointer (`*mut u_char`) allocated in the given nginx memory pool.
///
/// # Safety
///
/// The caller must provide a valid pointer to the memory pool.
pub unsafe fn bytes_to_uchar(pool: *mut ngx_pool_t, data: &[u8]) -> Option<*mut u_char> {
    let ptr: *mut u_char = ngx_pnalloc(pool, data.len()) as _;
    if ptr.is_null() {
        return None;
    }
    copy_nonoverlapping(data.as_ptr(), ptr, data.len());
    Some(ptr)
}

/// Convert a string slice (`&str`) to a raw pointer (`*mut u_char`) allocated in the given nginx memory pool.
///
/// # Arguments
///
/// * `pool` - A pointer to the nginx memory pool (`ngx_pool_t`).
/// * `data` - The string slice to convert to a raw pointer.
///
/// # Safety
/// This function is marked as unsafe because it involves raw pointer manipulation and direct memory allocation using `ngx_pnalloc`.
///
/// # Returns
/// A raw pointer (`*mut u_char`) to the allocated memory containing the converted string data.
///
/// # Example
/// ```rust,ignore
/// let pool: *mut ngx_pool_t = ...; // Obtain a pointer to the nginx memory pool
/// let data: &str = "example"; // The string to convert
/// let ptr = str_to_uchar(pool, data);
/// ```
pub unsafe fn str_to_uchar(pool: *mut ngx_pool_t, data: &str) -> *mut u_char {
    let ptr: *mut u_char = ngx_pnalloc(pool, data.len()) as _;
    debug_assert!(!ptr.is_null());
    copy_nonoverlapping(data.as_ptr(), ptr, data.len());
    ptr
}

impl ngx_str_t {
    /// Returns the contents of this `ngx_str_t` as a byte slice.
    ///
    /// The returned slice will **not** contain the optional nul terminator that `ngx_str_t.data`
    /// may have.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        if self.is_empty() {
            &[]
        } else {
            // SAFETY: `ngx_str_t` with non-zero len must contain a valid correctly aligned pointer
            unsafe { slice::from_raw_parts(self.data, self.len) }
        }
    }

    /// Returns the contents of this `ngx_str_t` as a mutable byte slice.
    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        if self.is_empty() {
            &mut []
        } else {
            // SAFETY: `ngx_str_t` with non-zero len must contain a valid correctly aligned pointer
            unsafe { slice::from_raw_parts_mut(self.data, self.len) }
        }
    }

    /// Returns `true` if the string has a length of 0.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Convert the nginx string to a string slice (`&str`).
    ///
    /// # Safety
    /// This function is marked as unsafe because it involves raw pointer manipulation.
    /// It assumes that the underlying `data` pointer is valid and points to a valid UTF-8 encoded string.
    ///
    /// # Panics
    /// This function panics if the `ngx_str_t` is not valid UTF-8.
    ///
    /// # Returns
    /// A string slice (`&str`) representing the nginx string.
    pub fn to_str(&self) -> &str {
        std::str::from_utf8(self.as_bytes()).unwrap()
    }

    /// Create an `ngx_str_t` instance from a byte slice.
    ///
    /// # Safety
    ///
    /// The caller must provide a valid pointer to a memory pool.
    pub unsafe fn from_bytes(pool: *mut ngx_pool_t, src: &[u8]) -> Option<Self> {
        bytes_to_uchar(pool, src).map(|data| Self { data, len: src.len() })
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
            len: data.len(),
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
            len: data.len(),
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
/// # use nginx_sys::*;
/// # unsafe fn example(pool: *mut ngx_pool_t, headers: *mut ngx_list_t) {
/// // Obtain a pointer to the nginx table entry
/// let table: *mut ngx_table_elt_t = ngx_list_push(headers).cast();
/// assert!(!table.is_null());
/// let key: &str = "key"; // The key to add
/// let value: &str = "value"; // The value to add
/// let result = add_to_ngx_table(table, pool, key, value);
/// # }
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
        table.key.len = key.len();
        table.key.data = str_to_uchar(pool, key);
        table.value.len = value.len();
        table.value.data = str_to_uchar(pool, value);
        table.lowcase_key = str_to_uchar(pool, String::from(key).to_ascii_lowercase().as_str());
    })
}
