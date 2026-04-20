use std::{
    fs::File,
    io::{self, Write},
    time::Duration,
};

use crate::event::{
    filter::{GraphicsSupportFilter, PrimaryDeviceAttributesFilter},
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
            InternalEvent::PrimaryDeviceAttributes(_) => return Ok(supported),
            InternalEvent::GraphicsSupportResponse => supported = true,
            _ => {}
        }
    }
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
    // Drain stale DA1 responses.
    while internal::poll(Some(Duration::ZERO), &PrimaryDeviceAttributesFilter)? {
        internal::read(&PrimaryDeviceAttributesFilter)?;
    }

    // DA1 request.
    const QUERY: &[u8] = b"\x1B[c";

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

    if !internal::poll(Some(timeout), &PrimaryDeviceAttributesFilter)? {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "The terminal sixel support could not be determined within a normal duration",
        ));
    }
    match internal::read(&PrimaryDeviceAttributesFilter)? {
        InternalEvent::PrimaryDeviceAttributes(attrs) => Ok(attrs.contains(&4)),
        _ => Ok(false),
    }
}
