//! # Graphics
//!
//! The `graphics` module provides functionality to query the terminal for
//! graphics-related capabilities.
//!
//! Use [`KittyQuery`] and [`SixelQuery`] with
//! [`QueryBatch`](crate::query::QueryBatch) to query graphics capabilities
//! from the terminal.

use std::io;

use crate::event::internal::InternalEvent;
use crate::query::TerminalQuery;

pub(crate) mod sys;

pub use sys::query_graphics_support;
pub use sys::query_sixel_support;

/// Query for [Kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/) support.
#[derive(Clone)]
pub struct KittyQuery;

impl TerminalQuery for KittyQuery {
    type Response = bool;

    fn query_bytes(&self) -> Vec<u8> {
        b"\x1B_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1B\\".to_vec()
    }

    fn matches(&self, event: &InternalEvent) -> bool {
        matches!(event, InternalEvent::GraphicsSupportResponse)
    }

    fn extract(&self, event: Option<InternalEvent>) -> io::Result<bool> {
        Ok(event.is_some())
    }
}

/// Query for [Sixel](https://vt100.net/docs/vt510-rm/DA1.html) graphics support.
///
/// Detected via DA1 attribute `4` in the terminal's primary device attributes response.
#[derive(Clone)]
pub struct SixelQuery;

impl TerminalQuery for SixelQuery {
    type Response = bool;

    fn query_bytes(&self) -> Vec<u8> {
        Vec::new() // Uses the DA1 sentinel that QueryBatch always appends.
    }

    fn matches(&self, event: &InternalEvent) -> bool {
        matches!(event, InternalEvent::PrimaryDeviceAttributes(_))
    }

    fn extract(&self, event: Option<InternalEvent>) -> io::Result<bool> {
        match event {
            Some(InternalEvent::PrimaryDeviceAttributes(attrs)) => Ok(attrs.contains(&4)),
            _ => Ok(false),
        }
    }
}
