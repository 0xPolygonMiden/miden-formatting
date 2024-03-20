#![doc = include_str!("../../README.md")]
#![no_std]

#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod hex;
pub mod prettier;
