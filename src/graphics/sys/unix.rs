use std::io;

use crate::graphics::{KittyQuery, SixelQuery};
use crate::query::QueryBatch;

/// Queries the terminal for Kitty graphics protocol support.
///
/// Sends a minimal graphics *query action* followed by a primary device
/// attributes request. A terminal that supports the protocol replies to both;
/// a terminal that does not only replies to the DA1 request.
///
/// See <https://sw.kovidgoyal.net/kitty/graphics-protocol/#querying-support-and-available-transmission-mediums>.
///
/// This function must be called while raw mode is enabled.
pub fn query_graphics_support() -> io::Result<bool> {
    let mut batch = QueryBatch::new();
    let h = batch.add(KittyQuery);
    batch.execute()?.get(&h)
}

/// Queries the terminal for Sixel graphics support.
///
/// Sends a primary device attributes request (`DA1`). Terminals that support
/// Sixel include attribute `4` in their response.
///
/// See <https://vt100.net/docs/vt510-rm/DA1.html> and
/// <https://www.vt100.net/docs/vt3xx-gp/chapter14.html>.
///
/// This function must be called while raw mode is enabled.
pub fn query_sixel_support() -> io::Result<bool> {
    let mut batch = QueryBatch::new();
    let h = batch.add(SixelQuery);
    batch.execute()?.get(&h)
}
