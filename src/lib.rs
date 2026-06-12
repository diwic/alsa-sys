#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![no_std]

use libc::{FILE, pid_t, pollfd, timespec, timeval};

#[cfg(feature = "use-bindgen")]
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[cfg(not(feature = "use-bindgen"))]
include!("generated.rs");
