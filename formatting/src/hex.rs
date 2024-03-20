//! This module provides various utilties for formatting values as hexadecimal bytes.

use alloc::string::String;
use core::fmt;

/// This trait represents a value that can be converted to a string of hexadecimal digits which
/// represent the raw byte encoding of that value.
///
/// This trait should only be implemented for types which can be decoded from the resulting string
/// of hexadecimal digits. It is not a strict requirement, but one that ensures that the
/// implementation is sane.
pub trait ToHex {
    /// Convert this value to a [String] containing the hexadecimal digits that correspond to the
    /// byte representation of this value.
    ///
    /// The resulting string should _not_ have a leading `0x` prefix. Use [ToHex::to_hex_with_prefix]
    /// if the prefix is needed.
    fn to_hex(&self) -> String;
    /// Same as [ToHex::to_hex], but ensures the output contains a leading `0x` prefix.
    fn to_hex_with_prefix(&self) -> String;
}

impl ToHex for [u8] {
    fn to_hex(&self) -> String {
        format!("{:x}", DisplayHex(self))
    }

    fn to_hex_with_prefix(&self) -> String {
        format!("{:#x}", DisplayHex(self))
    }
}

impl<'a> ToHex for DisplayHex<'a> {
    fn to_hex(&self) -> String {
        format!("{:x}", self)
    }

    fn to_hex_with_prefix(&self) -> String {
        format!("{:#x}", self)
    }
}

/// Construct a [String] containing the hexadecimal representation of `bytes`
#[inline]
pub fn to_hex(bytes: impl AsRef<[u8]>) -> String {
    bytes.as_ref().to_hex()
}

/// A display helper for formatting a slice of bytes as hex
/// with different options using Rust's builtin format language
pub struct DisplayHex<'a>(pub &'a [u8]);

impl<'a> DisplayHex<'a> {
    /// Display the underlying bytes of `item` as hexadecimal digits
    #[inline]
    pub fn new<'b: 'a, T>(item: &'b T) -> Self
    where
        T: AsRef<[u8]>,
    {
        Self(item.as_ref())
    }
}

impl<'a> fmt::Display for DisplayHex<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(self, f)
    }
}

impl<'a> fmt::LowerHex for DisplayHex<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            f.write_str("0x")?;
        }
        for byte in self.0.iter() {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl<'a> crate::prettier::PrettyPrint for DisplayHex<'a> {
    fn render(&self) -> crate::prettier::Document {
        crate::prettier::text(format!("{:#x}", self))
    }
}
