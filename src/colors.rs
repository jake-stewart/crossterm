//! # Colors
//!
//! The `colors` module provides functionality to query the terminal for colors
//! and color-related capabilities.
//!
//! Use [`ColorQuery`] and [`ColorSchemeQuery`] with
//! [`QueryBatch`](crate::graphics::QueryBatch) to query colors from the terminal.
//!
//! Color scheme *change* notifications are delivered as
//! [`Event::ColorSchemeChanged`](crate::event::Event::ColorSchemeChanged) when
//! [`EnableColorSchemeDetection`](crate::event::EnableColorSchemeDetection) is active.

#[cfg(unix)]
use std::io;
#[cfg(unix)]
use crate::event::internal::InternalEvent;
#[cfg(unix)]
use crate::query::TerminalQuery;

/// Terminal color type, used in queries and responses.
///
/// `Palette(n)` uses OSC 4. The remaining variants use OSC 10..=19.
#[derive(Debug, PartialOrd, PartialEq, Hash, Clone, Copy, Eq)]
pub enum ColorType {
    Palette(u8),
    Foreground,
    Background,
    Cursor,
    PointerForeground,
    PointerBackground,
    TektronixForeground,
    TektronixBackground,
    HighlightBackground,
    TektronixCursor,
    HighlightForeground,
}

impl ColorType {
    /// Maps an OSC number (10..=19) to the corresponding `ColorType` variant.
    pub(crate) fn from_osc_number(n: u8) -> Option<Self> {
        match n {
            10 => Some(Self::Foreground),
            11 => Some(Self::Background),
            12 => Some(Self::Cursor),
            13 => Some(Self::PointerForeground),
            14 => Some(Self::PointerBackground),
            15 => Some(Self::TektronixForeground),
            16 => Some(Self::TektronixBackground),
            17 => Some(Self::HighlightBackground),
            18 => Some(Self::TektronixCursor),
            19 => Some(Self::HighlightForeground),
            _ => None,
        }
    }

    /// Returns the OSC number for this color type.
    pub(crate) fn osc_number(&self) -> u8 {
        match self {
            Self::Palette(_) => 4,
            Self::Foreground => 10,
            Self::Background => 11,
            Self::Cursor => 12,
            Self::PointerForeground => 13,
            Self::PointerBackground => 14,
            Self::TektronixForeground => 15,
            Self::TektronixBackground => 16,
            Self::HighlightBackground => 17,
            Self::TektronixCursor => 18,
            Self::HighlightForeground => 19,
        }
    }
}

/// The terminal's color scheme preference (dark or light).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialOrd, Ord, PartialEq, Hash, Clone, Copy, Eq)]
pub enum ColorScheme {
    Dark,
    Light,
}

/// A parsed color response from the terminal.
#[derive(Debug, PartialOrd, PartialEq, Hash, Clone, Eq)]
pub(crate) struct ColorEntry {
    pub color_type: ColorType,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Query for an individual terminal color.
///
/// Returns the color as `(r, g, b)`. Use with [`QueryBatch`](crate::query::QueryBatch).
#[cfg(unix)]
#[derive(Clone)]
pub struct ColorQuery(pub ColorType);

#[cfg(unix)]
impl TerminalQuery for ColorQuery {
    type Response = (u8, u8, u8);

    fn query_bytes(&self) -> Vec<u8> {
        let n = self.0.osc_number();
        match self.0 {
            ColorType::Palette(index) => format!("\x1B]{n};{index};?\x1B\\").into_bytes(),
            _ => format!("\x1B]{n};?\x1B\\").into_bytes(),
        }
    }

    fn matches(&self, event: &InternalEvent) -> bool {
        matches!(event, InternalEvent::ColorResponse(e) if e.color_type == self.0)
    }

    fn extract(&self, event: Option<InternalEvent>) -> io::Result<(u8, u8, u8)> {
        match event {
            Some(InternalEvent::ColorResponse(e)) => Ok((e.r, e.g, e.b)),
            None => Err(io::Error::new(
                io::ErrorKind::Other,
                "terminal did not respond with color",
            )),
            _ => unreachable!(),
        }
    }
}

/// Query for the terminal's color scheme (dark or light mode).
///
/// Use with [`QueryBatch`](crate::query::QueryBatch).
#[cfg(unix)]
#[derive(Clone)]
pub struct ColorSchemeQuery;

#[cfg(unix)]
impl TerminalQuery for ColorSchemeQuery {
    type Response = ColorScheme;

    fn query_bytes(&self) -> Vec<u8> {
        b"\x1B[?996n".to_vec()
    }

    fn matches(&self, event: &InternalEvent) -> bool {
        matches!(event, InternalEvent::ColorSchemeResponse(_))
    }

    fn extract(&self, event: Option<InternalEvent>) -> io::Result<ColorScheme> {
        match event {
            Some(InternalEvent::ColorSchemeResponse(s)) => Ok(s),
            None => Err(io::Error::new(
                io::ErrorKind::Other,
                "terminal did not respond with color scheme",
            )),
            _ => unreachable!(),
        }
    }
}
