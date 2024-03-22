#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]
#![no_std]

#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod hex;
pub mod prettier;
