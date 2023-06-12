use crate::ffi::ngx_uint_t;
use std::{
    mem,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
};

/// SubrequestFlags is a bitmask for NGINX subrequest control.
/// Refer to https://nginx.org/en/docs/dev/development_guide.html#http_subrequests for more
/// details.
///
/// The following flags are available:
/// None: Zero value of the subrequest flag.
/// InMemory - Output is not sent to the client, but rather stored in memory. The flag only affects subrequests which are processed by one of the proxying modules. After a subrequest is finalized its output is available in r->out of type ngx_buf_t.
/// Waited - The subrequest's done flag is set even if the subrequest is not active when it is finalized. This subrequest flag is used by the SSI filter.
/// Clone - The subrequest is created as a clone of its parent. It is started at the same location and proceeds from the same phase as the parent request.
/// Background - The subrequest operates in the background (useful for background cache updates), this type of subrequest does not block any other subrequests or the main request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum SubrequestFlags {
    /// None: Zero value of the subrequest flag.
    None = 0,
    // unused = 1 ngx_http_request.h:65
    /// InMemory - Output is not sent to the client, but rather stored in memory. The flag only affects subrequests which are processed by one of the proxying modules. After a subrequest is finalized its output is available in r->out of type ngx_buf_t.
    InMemory = 2,
    /// Waited - The subrequest's done flag is set even if the subrequest is not active when it is finalized. This subrequest flag is used by the SSI filter.
    Waited = 4,
    /// Clone - The subrequest is created as a clone of its parent. It is started at the same location and proceeds from the same phase as the parent request.
    Clone = 8,
    /// Background - The subrequest operates in the background (useful for background cache updates), this type of subrequest does not block any other subrequests or the main request.
    Background = 16,
}

impl BitAnd for SubrequestFlags {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        unsafe { mem::transmute(self as u32 & rhs as u32) }
    }
}

impl BitAndAssign for SubrequestFlags {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl BitOr for SubrequestFlags {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        unsafe { mem::transmute(self as u32 | rhs as u32) }
    }
}

impl BitOrAssign for SubrequestFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl BitXor for SubrequestFlags {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self {
        unsafe { std::mem::transmute(self as u32 ^ rhs as u32) }
    }
}

impl BitXorAssign for SubrequestFlags {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl Not for SubrequestFlags {
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        unsafe { std::mem::transmute(!(self as u32)) }
    }
}

impl From<SubrequestFlags> for u32 {
    #[inline]
    fn from(flags: SubrequestFlags) -> Self {
        flags as u32
    }
}

impl From<SubrequestFlags> for ngx_uint_t {
    #[inline]
    fn from(flags: SubrequestFlags) -> Self {
        flags as ngx_uint_t
    }
}

impl SubrequestFlags {
    /// Tests if flag(s) are set on the SubrequestFlags.
    #[inline]
    pub fn has_flag(&self, flag: SubrequestFlags) -> bool {
        (*self as u32 & flag as u32) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_and() {
        let mut flag: SubrequestFlags = SubrequestFlags::Background;
        assert_eq!(flag.has_flag(SubrequestFlags::Background), true);

        let test_flag = flag & SubrequestFlags::InMemory;
        assert_eq!(test_flag, SubrequestFlags::None);
        assert_eq!(test_flag.has_flag(SubrequestFlags::Background), false);
        assert_eq!(test_flag.has_flag(SubrequestFlags::InMemory), false);

        flag &= SubrequestFlags::Clone;
        assert_eq!(flag, SubrequestFlags::None);
        assert_eq!(flag.has_flag(SubrequestFlags::Clone), false);
        assert_eq!(flag.has_flag(SubrequestFlags::Background), false);
    }

    #[test]
    fn test_or() {
        let mut flag: SubrequestFlags = SubrequestFlags::Background;
        assert_eq!(flag.has_flag(SubrequestFlags::Background), true);

        let test_flag = flag | SubrequestFlags::InMemory;
        assert_eq!(test_flag as u32, 18);
        assert_eq!(test_flag.has_flag(SubrequestFlags::Background), true);
        assert_eq!(test_flag.has_flag(SubrequestFlags::InMemory), true);
        assert_eq!(test_flag.has_flag(SubrequestFlags::Clone), false);

        flag |= SubrequestFlags::Clone;
        assert_eq!(flag as u32, 24);
        assert_eq!(flag.has_flag(SubrequestFlags::Background), true);
        assert_eq!(flag.has_flag(SubrequestFlags::Clone), true);
        assert_eq!(flag.has_flag(SubrequestFlags::InMemory), false);
    }

    #[test]
    fn test_xor() {
        let mut flag: SubrequestFlags = SubrequestFlags::Background | SubrequestFlags::InMemory;
        assert_eq!(flag as u32, 18);

        let test_flag = flag ^ SubrequestFlags::Background;
        assert_eq!(test_flag, SubrequestFlags::InMemory);
        assert_eq!(test_flag.has_flag(SubrequestFlags::Background), false);
        assert_eq!(test_flag.has_flag(SubrequestFlags::Clone), false);
        assert_eq!(test_flag.has_flag(SubrequestFlags::InMemory), true);

        flag ^= SubrequestFlags::Background;
        assert_eq!(flag, SubrequestFlags::InMemory);
        assert_eq!(flag.has_flag(SubrequestFlags::Background), false);
        assert_eq!(flag.has_flag(SubrequestFlags::Clone), false);
        assert_eq!(flag.has_flag(SubrequestFlags::InMemory), true);

        flag ^= SubrequestFlags::Clone;
        assert_eq!(flag as u32, 10);
        assert_eq!(flag.has_flag(SubrequestFlags::Background), false);
        assert_eq!(flag.has_flag(SubrequestFlags::Clone), true);
        assert_eq!(flag.has_flag(SubrequestFlags::InMemory), true);
    }

    #[test]
    fn test_not() {
        let flag: SubrequestFlags = SubrequestFlags::Background | SubrequestFlags::InMemory;
        assert_eq!(flag as u32, 18);

        let test_flag: SubrequestFlags = flag & !SubrequestFlags::Background;
        assert_eq!(test_flag, SubrequestFlags::InMemory);

        assert_eq!(!SubrequestFlags::InMemory as i32, -3);
        assert_eq!(!SubrequestFlags::Waited as i32, -5);
        assert_eq!(!SubrequestFlags::Clone as i32, -9);
        assert_eq!(!SubrequestFlags::Background as i32, -17);
    }
}
