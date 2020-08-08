#![feature(
    box_syntax,
    const_fn,
    crate_visibility_modifier,
    exclusive_range_pattern,
    in_band_lifetimes,
    specialization,
    stmt_expr_attributes
)]
#![cfg_attr(feature = "asm", feature(llvm_asm))]

#[macro_use]
pub mod macros;

pub mod cmp;
pub mod collections;
pub mod console;
pub mod crypto;
pub mod ffi;
pub mod fmt;
pub mod fs;
pub mod io;
pub mod math;
pub mod os;
pub mod path;
pub mod random;
pub mod str;
pub mod sync;
pub mod time;

mod core;

pub use self::core::*;
