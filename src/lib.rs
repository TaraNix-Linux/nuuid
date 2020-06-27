//! Create and use UUID's
#![cfg_attr(not(test), no_std)]
use bitvec::prelude::*;
use core::{
    convert::TryInto,
    fmt::{Error, Result as FmtResult, Write},
    str::FromStr,
};

/// A 16 byte with the UUID.
pub type Bytes = [u8; 16];

struct BytesWrapper<'a> {
    bytes: &'a mut [u8],
    offset: usize,
}

impl<'a> BytesWrapper<'a> {
    fn new(bytes: &'a mut [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn into_inner(self) -> &'a mut [u8] {
        self.bytes
    }
}

impl<'a> Write for BytesWrapper<'a> {
    fn write_str(&mut self, s: &str) -> FmtResult {
        if (self.bytes.len() - self.offset) < s.len() {
            return Err(Error);
        }
        self.bytes[self.offset..][..s.len()].copy_from_slice(s.as_bytes());
        self.offset += s.len();
        Ok(())
    }
}

/// UUID Variants
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Variant {
    /// Reserved for NCS backward compatibility.
    Ncs,

    /// RFC 4122 conforming UUID's.
    Rfc4122,

    /// Reserved for legacy Microsoft backward compatibility.
    Microsoft,

    /// Reserved for the future.
    Reserved,
}

/// UUID Version
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Version {
    /// Version 1, time based.
    Time,

    /// Version 2, DCE Security.
    Dce,

    /// Version 3, MD5 name based.
    Md5,

    /// Version 4, random.
    Random,

    /// Version 5, SHA-1 name based.
    Sha1,

    /// Special case for the nil UUID.
    Nil,
}

/// Error parsing UUID
#[derive(Debug)]
pub struct ParseUuidError;

/// Universally Unique Identifier, or UUID.
///
/// This type is `repr(transparent)` and guaranteed to have the same layout
/// as `[u8; 16]`.
///
/// UUID fields **always** laid out MSB, or big-endian.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Uuid(Bytes);

impl Uuid {
    /// The special Nil UUID, where all bits are set to zero.
    pub const fn nil() -> Self {
        Uuid([0; 16])
    }

    /// Create a UUID from bytes.
    pub const fn from_bytes(bytes: Bytes) -> Self {
        Self(bytes)
    }

    /// Return the UUID as it's bytes.
    pub const fn to_bytes(self) -> Bytes {
        self.0
    }

    /// Create a UUID from mixed-endian bytes.
    ///
    /// # Note
    ///
    /// This is primarily for compatibility with legacy version 2 UUID's,
    /// which use a mixed-endian format where the
    /// first three fields are little-endian.
    pub fn from_bytes_me(mut bytes: Bytes) -> Self {
        bytes[0..4].reverse();
        bytes[4..6].reverse();
        bytes[6..8].reverse();
        Self(bytes)
    }

    /// Return the UUID as mixed-endian bytes.
    ///
    /// See [`Uuid::from_bytes_le`] for details.
    pub fn to_bytes_me(self) -> Bytes {
        let mut bytes = self.to_bytes();
        bytes[0..4].reverse();
        bytes[4..6].reverse();
        bytes[6..8].reverse();
        bytes
    }

    /// Returns true if the UUID is nil.
    pub fn is_nil(self) -> bool {
        self.0 == Self::nil().0
    }

    /// The UUID Variant
    pub fn variant(self) -> Variant {
        let bits = &self.0[8].bits::<Msb0>()[..3];
        match (bits[0], bits[1], bits[2]) {
            (true, true, true) => Variant::Reserved,
            (true, true, false) => Variant::Microsoft,
            (true, false, ..) => Variant::Rfc4122,
            (false, ..) => Variant::Ncs,
        }
    }

    /// The UUID Variant
    ///
    /// # Panics
    ///
    /// - If the version is invalid
    pub fn version(self) -> Version {
        let bits = &self.0[6].bits::<Msb0>()[..4];
        match (bits[0], bits[1], bits[2], bits[3]) {
            (false, false, false, false) => Version::Nil,
            (false, false, false, true) => Version::Time,
            (false, false, true, false) => Version::Dce,
            (false, false, true, true) => Version::Md5,
            (false, true, false, false) => Version::Random,
            (false, true, false, true) => Version::Sha1,
            _ => panic!("Invalid version"),
        }
    }

    /// Write UUID as a string into `buf`, and returns it as a string.
    pub fn to_string(self, buf: &mut [u8; 36]) -> &str {
        let bytes = self.to_bytes();
        let time_low = u32::from_be_bytes(bytes[..4].try_into().unwrap());
        let time_mid = u16::from_be_bytes(bytes[4..6].try_into().unwrap());
        let time_hi_and_version = u16::from_be_bytes(bytes[6..8].try_into().unwrap());
        let clock_seq_hi_and_reserved = u8::from_be_bytes(bytes[8..9].try_into().unwrap());
        let clock_seq_low = u8::from_be_bytes(bytes[9..10].try_into().unwrap());
        let mut node = [0; 8];
        // Leading zeros, and last 48 bits/6 bytes
        node[2..].copy_from_slice(&bytes[10..16]);
        let node = u64::from_be_bytes(node);
        let mut buf = BytesWrapper::new(&mut buf[..]);
        write!(
            buf,
            "{:x}-{:x}-{:x}-{:x}{:x}-{:x}",
            time_low, time_mid, time_hi_and_version, clock_seq_hi_and_reserved, clock_seq_low, node
        )
        .expect("BUG: Couldn't write UUID");
        core::str::from_utf8(buf.into_inner()).expect("BUG: Invalid UTF")
    }
}

impl FromStr for Uuid {
    type Err = ParseUuidError;

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UUID_V4: &str = "662aa7c7-7598-4d56-8bcc-a72c30f998a2";
    const RAW: [u8; 16] = [
        102, 42, 167, 199, 117, 152, 77, 86, 139, 204, 167, 44, 48, 249, 152, 162,
    ];

    #[test]
    fn string() {
        let uuid = Uuid::from_bytes(RAW);
        let mut buf = [0; 36];
        let s = uuid.to_string(&mut buf);
        println!("UUID: `{}`", s);
        assert_eq!(s, UUID_V4, "UUID strings didn't match");
    }

    #[test]
    fn endian() {
        let uuid_be = Uuid::from_bytes(RAW);
        assert_eq!(uuid_be.version(), Version::Random);
        assert_eq!(uuid_be.variant(), Variant::Rfc4122);

        let uuid_le = Uuid::from_bytes_me(uuid_be.to_bytes_me());
        assert_eq!(uuid_le.version(), Version::Random);
        assert_eq!(uuid_le.variant(), Variant::Rfc4122);
    }

    #[test]
    fn info() {
        let uuid = Uuid::from_bytes(RAW);
        assert_eq!(uuid.version(), Version::Random);
        assert_eq!(uuid.variant(), Variant::Rfc4122);
    }
}
