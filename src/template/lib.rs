#![cfg_attr(not(feature = "std"), no_std, feature(core_c_str), feature(core_ffi_c))]

mod gl;
pub use gl::*;
