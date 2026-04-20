//! This module provides platform related functions.

#[cfg(unix)]
pub use self::unix::query_graphics_support;
#[cfg(unix)]
pub use self::unix::query_sixel_support;
#[cfg(windows)]
pub use self::windows::query_graphics_support;
#[cfg(windows)]
pub use self::windows::query_sixel_support;

#[cfg(unix)]
pub(crate) mod unix;
#[cfg(windows)]
pub(crate) mod windows;
