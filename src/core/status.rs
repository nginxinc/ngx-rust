use crate::ffi::*;
use std::fmt;

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub struct Status(pub ngx_int_t);

impl Status {
    pub fn is_ok(&self) -> bool {
        self == &Status::NGX_OK
    }
}

impl fmt::Debug for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl From<Status> for ngx_int_t {
    fn from(val: Status) -> Self {
        val.0
    }
}

macro_rules! ngx_codes {
    (
        $(
            $(#[$docs:meta])*
            ($konst:ident);
        )+
    ) => {
        impl Status {
        $(
            $(#[$docs])*
            pub const $konst: Status = Status($konst as ngx_int_t);
        )+

        }
    }
}

ngx_codes! {
    (NGX_OK);
    (NGX_ERROR);
    (NGX_AGAIN);
    (NGX_BUSY);
    (NGX_DONE);
    (NGX_DECLINED);
    (NGX_ABORT);
}
pub const NGX_CONF_ERROR: *const () = -1isize as *const ();
// pub const CONF_OK: Status = Status(NGX_CONF_OK as ngx_int_t);
