use crate::ffi::*;

#[repr(transparent)]
pub struct Event(ngx_event_t);
impl Event {
    pub fn add_timer(&mut self, timer: ngx_msec_t) {
        let key: ngx_msec_int_t = unsafe {ngx_current_msec as isize + timer as isize};
        if self.0.timer_set() == 0 {
            /* FROM NGX:
             * Use a previous timer value if difference between it and a new
             * value is less than NGX_TIMER_LAZY_DELAY milliseconds: this allows
             * to minimize the rbtree operations for fast connections.
             */
            let diff = key - self.0.timer.key as ngx_msec_int_t;
            if diff.abs() < NGX_TIMER_LAZY_DELAY as isize {
                // TODO add debugging macro
                return;
            }

            self.del_timer();
        }

        self.0.timer.key = key as ngx_msec_t;
        // TODO add debugging macro
        unsafe {
            ngx_rbtree_insert(
                &mut ngx_event_timer_rbtree as *mut _,
                &mut self.0.timer as *mut _,
            );
        }

        self.0.set_timer_set(1);
    }

    pub fn del_timer(&mut self) {
        unsafe {
            ngx_rbtree_delete(
                &mut ngx_event_timer_rbtree as *mut _,
                &mut self.0.timer as *mut _,
            );
        }

        self.0.set_timer_set(0);
    }
}

impl From<*mut ngx_event_t> for &mut Event {
    fn from(evt: *mut ngx_event_t) -> Self {
        unsafe {&mut *evt.cast::<Event>()}
    }
}

