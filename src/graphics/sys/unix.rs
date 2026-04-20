use std::{
    fs::File,
    io::{self, Write},
    time::Duration,
};

use crate::event::{
    filter::GraphicsSupportFilter,
    internal::{self, InternalEvent},
};

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
    // Drain stale responses.
    while internal::poll(Some(Duration::ZERO), &GraphicsSupportFilter)? {
        internal::read(&GraphicsSupportFilter)?;
    }

    // APC _G i=31,s=1,v=1,a=q,t=d,f=24 ; AAAA ST  then  CSI c (DA1).
    // A supporting terminal replies to the APC *before* the DA1 response.
    const QUERY: &[u8] = b"\x1B_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1B\\\x1B[c";

    let result = File::options()
        .write(true)
        .open("/dev/tty")
        .and_then(|mut file| {
            file.write_all(QUERY)?;
            file.flush()
        });
    if result.is_err() {
        let mut stdout = io::stdout();
        stdout.write_all(QUERY)?;
        stdout.flush()?;
    }

    let timeout = Duration::from_secs(2);
    let mut supported = false;

    loop {
        if !internal::poll(Some(timeout), &GraphicsSupportFilter)? {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "The terminal graphics support could not be determined within a normal duration",
            ));
        }
        match internal::read(&GraphicsSupportFilter)? {
            InternalEvent::PrimaryDeviceAttributes => return Ok(supported),
            InternalEvent::GraphicsSupportResponse => supported = true,
            _ => {}
        }
    }
}
