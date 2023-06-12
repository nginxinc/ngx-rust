use crate::ffi::*;
use crate::ngx_log_debug_mask;

/// Wrapper struct for an `ngx_event_t` pointer, provides an interface into timer methods.
#[repr(transparent)]
pub struct Event(pub ngx_event_t);

impl Event {
    #[inline]
    fn ident(&self) -> i32 {
        let conn = self.0.data as *const ngx_connection_t;
        unsafe { (*conn).fd }
    }

    /// Adds a timer to this event. Argument `timer` is in milliseconds.
    pub fn add_timer(&mut self, timer_msec: ngx_msec_t) {
        let key: ngx_msec_int_t = unsafe { ngx_current_msec as isize + timer_msec as isize };
        if self.0.timer_set() != 0 {
            /* FROM NGX:
             * Use a previous timer value if difference between it and a new
             * value is less than NGX_TIMER_LAZY_DELAY milliseconds: this allows
             * to minimize the rbtree operations for fast connections.
             */
            let diff = key - self.0.timer.key as ngx_msec_int_t;
            if diff.abs() < NGX_TIMER_LAZY_DELAY as isize {
                ngx_log_debug_mask!(
                    NGX_LOG_DEBUG_EVENT,
                    self.0.log,
                    "event time: {}, old: {:?}, new: {:?}",
                    self.ident(),
                    self.0.timer.key,
                    key
                );
                return;
            }

            self.del_timer();
        }

        self.0.timer.key = key as ngx_msec_t;
        ngx_log_debug_mask!(
            NGX_LOG_DEBUG_EVENT,
            self.0.log,
            "event time: {}, old: {:?}, new: {:?}",
            self.ident(),
            self.0.timer.key,
            key
        );
        unsafe {
            ngx_rbtree_insert(&mut ngx_event_timer_rbtree as *mut _, &mut self.0.timer as *mut _);
        }

        self.0.set_timer_set(1);
    }

    /// Deletes an associated timer from this event.
    pub fn del_timer(&mut self) {
        ngx_log_debug_mask!(
            NGX_LOG_DEBUG_EVENT,
            self.0.log,
            "event timer del: {}:{:?}",
            self.ident(),
            self.0.timer.key
        );
        unsafe {
            ngx_rbtree_delete(&mut ngx_event_timer_rbtree as *mut _, &mut self.0.timer as *mut _);
        }

        self.0.set_timer_set(0);
    }

    /// Add event to processing queue. Translated from ngx_post_event macro.
    ///
    /// # Safety
    /// This function is marked unsafe because it dereferences a raw pointer. The pointer (queue)
    /// MUST NOT be null to satisfy its contract, will panic with null input.
    ///
    /// # Panics
    /// Panics if the given queue is null.
    pub unsafe fn post_to_queue(&mut self, queue: *mut ngx_queue_t) {
        assert!(!queue.is_null(), "queue is empty");
        if self.0.posted() == 0 {
            self.0.set_posted(1);
            // translated from ngx_queue_insert_tail macro
            self.0.queue.prev = (*queue).prev;
            (*self.0.queue.prev).next = &self.0.queue as *const _ as *mut _;
            self.0.queue.next = queue;
            (*queue).prev = &self.0.queue as *const _ as *mut _;
        }
    }

    /// new_for_request creates an new Event (ngx_event_t) from the Request pool.
    ///
    /// # Safety
    /// This function is marked as unsafe because it involves dereferencing a raw pointer memory
    /// allocation from the underlying Nginx pool allocator.
    ///
    /// # Returns
    /// An `Option<&mut Event>` representing the result of the allocation. `Some(&mut Event)`
    /// indicates successful allocation, while `None` indicates a null Event.
    pub unsafe fn new_for_request(req: &mut crate::http::Request) -> Option<&mut Event> {
        Some(&mut *(req.pool().alloc(std::mem::size_of::<ngx_event_t>()) as *mut Event))
    }
}

impl From<*mut ngx_event_t> for &mut Event {
    fn from(evt: *mut ngx_event_t) -> Self {
        unsafe { &mut *evt.cast::<Event>() }
    }
}

impl From<&mut Event> for *mut ngx_event_t {
    fn from(val: &mut Event) -> Self {
        &mut val.0 as *mut ngx_event_t
    }
}
