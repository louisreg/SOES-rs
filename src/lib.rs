#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

#[cfg(feature = "std")]
extern crate std;

pub mod bindings;

pub mod soes;

pub use soes::*;
pub mod drivers;
pub mod esc_driver;

/// Simple Wrapper for tests (??)
pub fn soes_version() -> u32 {
    1
}
