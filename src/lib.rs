#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![no_std]

use libc::{FILE, pid_t, pollfd, timeval};

#[cfg(not(alsa_sys_time64))]
pub use libc::timespec;

// Mirrors glibc's 32-bit time64 struct timespec; see probe_time64 in build.rs.
#[cfg(alsa_sys_time64)]
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature = "extra_traits", derive(PartialEq, Eq, Hash))]
pub struct timespec {
    pub tv_sec: i64,
    #[cfg(target_endian = "little")]
    pub tv_nsec: i32,
    #[cfg(target_endian = "little")]
    __pad: i32,
    #[cfg(target_endian = "big")]
    __pad: i32,
    #[cfg(target_endian = "big")]
    pub tv_nsec: i32,
}

#[cfg(feature = "use-bindgen")]
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[cfg(not(feature = "use-bindgen"))]
include!("generated.rs");
