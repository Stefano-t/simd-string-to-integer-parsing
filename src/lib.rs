#![feature(stdsimd)]

pub mod fallback;
pub mod sse2;

pub use sse2::parse_integer;
