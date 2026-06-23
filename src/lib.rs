#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![no_std]

use core::mem::zeroed;

use libc::{FILE, pid_t, pollfd, timeval};

#[derive(Copy, Clone)]
#[repr(C)]
pub union timespec {
    t: libc::timespec,
    t64: [i64; 2],
}

#[cfg(not(target_pointer_width = "32"))]
impl timespec {
    #[inline(always)]
    pub fn tv_sec(&self) -> i64 { unsafe { self.t.tv_sec as i64 } }
    #[inline(always)]
    pub fn tv_nsec(&self) -> i64 { unsafe { self.t.tv_nsec as i64 } }
}

#[cfg(target_pointer_width = "32")]
impl timespec {
    unsafe fn is64(&self) -> bool { self.t64[1] != 0 }
    pub fn tv_sec(&self) -> i64 { unsafe { if self.is64() { self.t64[0] } else { self.t.tv_sec as i64 } } }
    pub fn tv_nsec(&self) -> i64 { unsafe { if self.is64() { self.t64[1] } else { self.t.tv_nsec as i64 } } }
}

impl Default for timespec {
    fn default() -> Self { unsafe { zeroed() } }
}

#[cfg(feature = "use-bindgen")]
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[cfg(not(feature = "use-bindgen"))]
include!("generated.rs");
