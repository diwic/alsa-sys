#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![no_std]

use libc::{FILE, pid_t, pollfd, timeval};

#[cfg(target_pointer_width = "64")]
#[derive(Copy, Clone, Eq, PartialEq, Default)]
#[repr(C)]
pub struct timespec([i64; 2]);

#[cfg(target_pointer_width = "64")]
impl timespec {
    #[inline(always)]
    pub fn tv_sec(&self) -> i64 { self.0[0] }
    #[inline(always)]
    pub fn tv_nsec(&self) -> i64 { self.0[1] }
}

#[cfg(target_pointer_width = "32")]
#[derive(Copy, Clone, Eq, PartialEq, Default)]
#[repr(C)]
pub struct timespec([i32; 4]);

#[cfg(target_pointer_width = "32")]
impl timespec {
    fn is64(&self) -> bool { self.0[2] != 0 || self.0[3] != 0 }
    pub fn tv_sec(&self) -> i64 { self.0[0] as i64 }
    pub fn tv_nsec(&self) -> i64 { if self.is64() { (self.0[2] + self.0[3]) as i64 } else { self.0[1] as i64 } }
}


#[cfg(feature = "use-bindgen")]
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[cfg(not(feature = "use-bindgen"))]
include!("generated.rs");
