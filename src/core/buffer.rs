use crate::ffi::*;

use std::slice;

pub trait Buffer {
    fn as_ngx_buf(&self) -> *const ngx_buf_t;

    fn as_ngx_buf_mut(&mut self) -> *mut ngx_buf_t;

    fn as_bytes(&self) -> &[u8] {
        let buf = self.as_ngx_buf();
        unsafe { slice::from_raw_parts((*buf).pos, self.len()) }
    }

    fn len(&self) -> usize {
        let buf = self.as_ngx_buf();
        unsafe {
            let pos = (*buf).pos;
            let last = (*buf).last;
            assert!(last >= pos);
            usize::wrapping_sub(last as _, pos as _)
        }
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn set_last_buf(&mut self, last: bool) {
        let buf = self.as_ngx_buf_mut();
        unsafe {
            (*buf).set_last_buf(if last { 1 } else { 0 });
        }
    }

    fn set_last_in_chain(&mut self, last: bool) {
        let buf = self.as_ngx_buf_mut();
        unsafe {
            (*buf).set_last_in_chain(if last { 1 } else { 0 });
        }
    }
}

pub trait MutableBuffer: Buffer {
    fn as_bytes_mut(&mut self) -> &mut [u8] {
        let buf = self.as_ngx_buf_mut();
        unsafe { slice::from_raw_parts_mut((*buf).pos, self.len()) }
    }
}

pub struct TemporaryBuffer(*mut ngx_buf_t);

impl TemporaryBuffer {
    pub fn from_ngx_buf(buf: *mut ngx_buf_t) -> TemporaryBuffer {
        assert!(!buf.is_null());
        TemporaryBuffer(buf)
    }
}

impl Buffer for TemporaryBuffer {
    fn as_ngx_buf(&self) -> *const ngx_buf_t {
        self.0
    }

    fn as_ngx_buf_mut(&mut self) -> *mut ngx_buf_t {
        self.0
    }
}

impl MutableBuffer for TemporaryBuffer {
    fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut((*self.0).pos, self.len()) }
    }
}

pub struct MemoryBuffer(*mut ngx_buf_t);

impl MemoryBuffer {
    pub fn from_ngx_buf(buf: *mut ngx_buf_t) -> MemoryBuffer {
        assert!(!buf.is_null());
        MemoryBuffer(buf)
    }
}

impl Buffer for MemoryBuffer {
    fn as_ngx_buf(&self) -> *const ngx_buf_t {
        self.0
    }

    fn as_ngx_buf_mut(&mut self) -> *mut ngx_buf_t {
        self.0
    }
}
