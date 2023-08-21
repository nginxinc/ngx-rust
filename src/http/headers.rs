use crate::ffi::{
    ngx_http_headers_in_t, ngx_http_headers_out_t, ngx_list_part_t, ngx_list_t, ngx_pool_t, ngx_table_elt_t,
    ngx_uint_t, u_char,
};
use crate::ngx_string;
use std::ffi::CString;
use std::fmt;
use std::ptr;

pub struct Header(*mut ngx_table_elt_t, *mut ngx_pool_t);

impl Header {
    pub fn set_value(&mut self, value: &str) {
        // we can use pool to allocate and then copy
        unsafe { self.0.as_mut() }.map(|table| {
            table.value.len = value.len() as _;
            table.value.data = crate::ffi::str_to_uchar(self.1, value);
        });

        // Alternative way is using CString and transfer ownership.
        // unsafe { self.0.as_mut() }.map(|table| {
        //     let c_value = CString::new(value).unwrap();
        //     // table.value.len = c_value.as_bytes().len();
        //     table.value.len = c_value.len();
        //     table.value.data = c_value.into_raw() as *mut u_char;
        // });
    }

    pub fn set_key(&mut self, key: &str) {
        // we can use pool to allocate and then copy
        unsafe { self.0.as_mut() }.map(|table| {
            table.key.len = key.len() as _;
            table.key.data = crate::ffi::str_to_uchar(self.1, key);
            table.lowcase_key = crate::ffi::str_to_uchar(self.1, key.to_lowercase().as_str());
        });
    }

    pub fn set_hash(&mut self, hash: ngx_uint_t) {
        unsafe { self.0.as_mut() }.map(|table| {
            table.hash = hash;
        });
    }

    pub fn value(&self) -> &str {
        unsafe {
            std::str::from_utf8(std::slice::from_raw_parts(
                unsafe { *self.0 }.value.data,
                unsafe { *self.0 }.value.len as usize,
            ))
            .unwrap()
        }
    }

    pub fn key(&self) -> &str {
        unsafe {
            std::str::from_utf8(std::slice::from_raw_parts(
                (*self.0).key.data,
                (*self.0).key.len as usize,
            ))
            .unwrap()
        }
    }
    pub fn lowercase_key(&self) -> Option<&str> {
        unsafe {
            let byte_slice = std::slice::from_raw_parts_mut(unsafe { *self.0 }.lowcase_key, unsafe { *self.0 }.key.len);
            if let Ok(utf8_str) = std::str::from_utf8(byte_slice) {
                Some(utf8_str)
            } else {
                // not a valid UTF-8, return None
                None
            }
        }
    }

    pub fn hash(&self) -> ngx_uint_t {
        unsafe { *self.0 }.hash
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.key(), self.value())
    }
}

/// Iterator for `ngx_list_t` types.
///
/// Implementes the std::iter::Iterator trait.
pub struct HeadersIterator {
    done: bool,
    part: *const ngx_list_part_t,
    h: *mut ngx_table_elt_t,
    i: ngx_uint_t,
    pool: *mut ngx_pool_t,
}

impl HeadersIterator {
    pub unsafe fn new(list: *const ngx_list_t) -> Self {
        let part: *const ngx_list_part_t = &(*list).part;
        let p = (*list).pool;
        Self {
            done: false,
            part,
            h: (*part).elts as *mut ngx_table_elt_t,
            i: 0,
            pool: p,
        }
    }
}

impl Iterator for HeadersIterator {
    type Item = Header;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.done {
                None
            } else {
                if self.i >= (*self.part).nelts {
                    if (*self.part).next.is_null() {
                        self.done = true;
                        return None;
                    }

                    // loop back
                    self.part = (*self.part).next;
                    self.h = (*self.part).elts as *mut ngx_table_elt_t;
                    self.i = 0;
                }

                let header: *mut ngx_table_elt_t = self.h.add(self.i);
                self.i += 1;
                Some(Header(header, self.pool))
            }
        }
    }
}
