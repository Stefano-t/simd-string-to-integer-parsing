#![feature(stdsimd)]

mod fallback;
mod sse2;

pub use sse2::parse_integer;
