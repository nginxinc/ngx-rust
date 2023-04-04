#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::all)]
#![allow(improper_ctypes)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::fmt;
use std::ptr::copy_nonoverlapping;
use std::slice;

pub fn str_to_uchar(pool: *mut ngx_pool_t, data: &str) -> *mut u_char {
    let ptr: *mut u_char = unsafe { ngx_palloc(pool, data.len() as _) as _ };
    unsafe {
        copy_nonoverlapping(data.as_ptr(), ptr, data.len());
    }
    ptr
}

impl ngx_str_t {
    // convert nginx string to str slice
    pub fn to_str(&self) -> &str {
        unsafe {
            let slice = slice::from_raw_parts(self.data, self.len as usize);
            return std::str::from_utf8(slice).unwrap();
        }
    }

    // get string
    pub fn to_string(&self) -> String {
        return String::from(self.to_str());
    }

    /// create from string
    pub fn from_string(pool: *mut ngx_pool_t, data: String) -> Self {
        ngx_str_t {
            data: str_to_uchar(pool, data.as_str()),
            len: data.len() as _,
        }
    }

    /// create from string
    pub fn from_str(pool: *mut ngx_pool_t, data: &str) -> Self {
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
        unsafe { slice::from_raw_parts(s.data, s.len as usize) }
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

pub fn add_to_ngx_table(table: *mut ngx_table_elt_t, pool: *mut ngx_pool_t, key: &str, value: &str) -> Option<()> {
    if table.is_null() {
        return None;
    }
    unsafe { table.as_mut() }.map(|table| {
        table.hash = 1;
        table.key.len = key.len() as _;
        table.key.data = str_to_uchar(pool, key);
        table.value.len = value.len() as _;
        table.value.data = str_to_uchar(pool, value);
        table.lowcase_key = str_to_uchar(pool, String::from(key).to_ascii_lowercase().as_str());
    })
}
