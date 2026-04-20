use std::io;

/// Queries the terminal for Kitty graphics protocol support.
///
/// Not supported on Windows; always returns an [`io::ErrorKind::Unsupported`] error.
pub fn query_graphics_support() -> io::Result<bool> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Querying graphics support is not implemented for the Windows API.",
    ))
}

/// Queries the terminal for Sixel graphics support.
///
/// Not supported on Windows; always returns an [`io::ErrorKind::Unsupported`] error.
pub fn query_sixel_support() -> io::Result<bool> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Querying sixel support is not implemented for the Windows API.",
    ))
}
