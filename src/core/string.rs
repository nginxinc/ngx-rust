use crate::ffi::*;

use std::borrow::Cow;
use std::cmp::Ordering;
use std::slice;
use std::str::{self, Utf8Error};

/// Static string initializer for [`ngx_str_t`].
///
/// The resulting byte string is always nul-terminated (just like a C string).
///
/// [`ngx_str_t`]: https://nginx.org/en/docs/dev/development_guide.html#string_overview
#[macro_export]
macro_rules! ngx_string {
    ($s:expr) => {{
        $crate::ffi::ngx_str_t {
            len: $s.len() as _,
            data: concat!($s, "\0").as_ptr() as *mut u8,
        }
    }};
}

/// Static empty string initializer for [`ngx_str_t`].
///
/// [`ngx_str_t`]: https://nginx.org/en/docs/dev/development_guide.html#string_overview
#[macro_export]
macro_rules! ngx_null_string {
    () => {
        $crate::ffi::ngx_str_t {
            len: 0,
            data: ::std::ptr::null_mut(),
        }
    };
}

/// Representation of a borrowed [Nginx string].
///
/// [Nginx string]: https://nginx.org/en/docs/dev/development_guide.html#string_overview
pub struct NgxStr([u_char]);

impl NgxStr {
    /// Create an [`NgxStr`] from an [`ngx_str_t`].
    ///
    /// [`ngx_str_t`]: https://nginx.org/en/docs/dev/development_guide.html#string_overview
    ///
    /// # Safety
    ///
    /// The caller has provided a valid `ngx_str_t` with a `data` pointer that points
    /// to range of bytes of at least `len` bytes, whose content remains valid and doesn't
    /// change for the lifetime of the returned `NgxStr`.
    pub unsafe fn from_ngx_str<'a>(str: ngx_str_t) -> &'a NgxStr {
        slice::from_raw_parts(str.data, str.len).into()
    }

    /// Converts the [`NgxStr`] to a byte slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use ngx::core::NgxStr;
    /// use ngx::ngx_string;
    ///
    /// let ngx_str = unsafe { NgxStr::from_ngx_str(ngx_string!("hello")) };
    /// let bytes = ngx_str.as_bytes();
    /// assert_eq!(bytes, &[104, 101, 108, 108, 111]); // "hello" in ASCII
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    /// Converts the Nginx string reference to a Rust string slice (`&str`).
    ///
    /// # Safety
    ///
    /// This operation is marked as `unsafe` because it involves converting raw bytes to a
    /// string slice. The safety of this operation depends on the validity of the underlying
    /// bytes and the proper lifetime management of `NgxStr`.
    ///
    /// It's important to ensure that the bytes in the `NgxStr` are valid UTF-8 and that the
    /// `NgxStr` instance is not used after its underlying data is deallocated.
    ///
    /// # Examples
    ///
    /// ```
    /// use ngx::core::NgxStr;
    /// use ngx::ngx_string;
    ///
    /// let ngx_str = unsafe { NgxStr::from_ngx_str(ngx_string!("hello")) };
    /// let rust_str: &str = ngx_str.as_str();
    /// assert_eq!(rust_str, "hello");
    /// ```
    pub fn as_str(&self) -> &str {
        // Safety: Converting the raw data to a string slice is unsafe, but as long
        // as the lifetime of NgxStr is properly managed, this should be safe.
        unsafe {
            // let slice = std::slice::from_raw_parts(self.0.data, self.0.len);
            std::str::from_utf8_unchecked(&self.0)
        }
    }

    /// Yields a `&str` slice if the [`NgxStr`] contains valid UTF-8.
    pub fn to_str(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(self.as_bytes())
    }

    /// Converts an [`NgxStr`] into a [`Cow<str>`], replacing invalid UTF-8 sequences.
    ///
    /// See [`String::from_utf8_lossy`].
    pub fn to_string_lossy(&self) -> Cow<str> {
        String::from_utf8_lossy(self.as_bytes())
    }

    /// Returns `true` if the [`NgxStr`] is empty, otherwise `false`.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Compare the NgxStr with another &str using case-insensitive UTF-8 comparison.
    pub fn cmp_ignore_case_utf8(&self, other: &str) -> Ordering {
        let self_slice = self.as_str().to_lowercase();
        let other_slice = other.to_lowercase();
        self_slice.cmp(&other_slice)
    }
}

impl From<&[u8]> for &NgxStr {
    fn from(bytes: &[u8]) -> Self {
        // SAFETY: An `NgxStr` is identical to a `[u8]` slice, given `u_char` is an alias for `u8`.
        unsafe { &*(bytes as *const [u8] as *const NgxStr) }
    }
}

impl From<&str> for &NgxStr {
    fn from(s: &str) -> Self {
        s.as_bytes().into()
    }
}

impl AsRef<[u8]> for NgxStr {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Default for &NgxStr {
    fn default() -> Self {
        // SAFETY: The null `ngx_str_t` is always a valid Nginx string.
        unsafe { NgxStr::from_ngx_str(ngx_null_string!()) }
    }
}
