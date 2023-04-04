use crate::core::buffer::{Buffer, MemoryBuffer, TemporaryBuffer};
use crate::ffi::*;

use std::os::raw::c_void;
use std::{mem, ptr};

pub struct Pool(*mut ngx_pool_t);

impl Pool {
    /// # Safety
    ///
    /// The caller has provided a valid `ngx_pool_t` that points to valid memory and is non-null.
    /// A null argument will assert and panic.
    pub unsafe fn from_ngx_pool(pool: *mut ngx_pool_t) -> Pool {
        assert!(!pool.is_null());
        Pool(pool)
    }

    pub fn create_buffer(&mut self, size: usize) -> Option<TemporaryBuffer> {
        let buf = unsafe { ngx_create_temp_buf(self.0, size) };
        if buf.is_null() {
            return None;
        }

        Some(TemporaryBuffer::from_ngx_buf(buf))
    }

    pub fn create_buffer_from_str(&mut self, str: &str) -> Option<TemporaryBuffer> {
        let mut buffer = self.create_buffer(str.len())?;
        unsafe {
            let mut buf = buffer.as_ngx_buf_mut();
            ptr::copy_nonoverlapping(str.as_ptr(), (*buf).pos, str.len());
            (*buf).last = (*buf).pos.add(str.len());
        }
        Some(buffer)
    }

    pub fn create_buffer_from_static_str(&mut self, str: &'static str) -> Option<MemoryBuffer> {
        let buf = self.calloc_type::<ngx_buf_t>();
        if buf.is_null() {
            return None;
        }

        // We cast away const, but buffers with the memory flag are read-only
        let start = str.as_ptr() as *mut u8;
        let end = unsafe { start.add(str.len()) };

        unsafe {
            (*buf).start = start;
            (*buf).pos = start;
            (*buf).last = end;
            (*buf).end = end;
            (*buf).set_memory(1);
        }

        Some(MemoryBuffer::from_ngx_buf(buf))
    }

    unsafe fn add_cleanup_for_value<T>(&mut self, value: *mut T) -> Result<(), ()> {
        let cln = ngx_pool_cleanup_add(self.0, 0);
        if cln.is_null() {
            return Err(());
        }
        (*cln).handler = Some(cleanup_type::<T>);
        (*cln).data = value as *mut c_void;

        Ok(())
    }

    pub fn alloc(&mut self, size: usize) -> *mut c_void {
        unsafe { ngx_palloc(self.0, size) }
    }

    pub fn alloc_type<T: Copy>(&mut self) -> *mut T {
        self.alloc(mem::size_of::<T>()) as *mut T
    }

    pub fn calloc(&mut self, size: usize) -> *mut c_void {
        unsafe { ngx_pcalloc(self.0, size) }
    }

    pub fn calloc_type<T: Copy>(&mut self) -> *mut T {
        self.calloc(mem::size_of::<T>()) as *mut T
    }

    pub fn allocate<T>(&mut self, value: T) -> *mut T {
        unsafe {
            let p = self.alloc(mem::size_of::<T>()) as *mut T;
            ptr::write(p, value);
            if self.add_cleanup_for_value(p).is_err() {
                ptr::drop_in_place(p);
                return ptr::null_mut();
            };
            p
        }
    }
}

unsafe extern "C" fn cleanup_type<T>(data: *mut c_void) {
    ptr::drop_in_place(data as *mut T);
}
