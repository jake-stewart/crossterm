//! # Graphics
//!
//! The `graphics` module provides functionality to query the terminal for
//! graphics-related capabilities.
//!
//! * [`query_graphics_support`] — detect whether the terminal supports the
//!   [Kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/).

pub(crate) mod sys;

pub use sys::query_graphics_support;
