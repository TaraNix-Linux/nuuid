//! Create and use UUID's
// #![cfg_attr(not(test), no_std)]
use bitvec::prelude::*;
use core::convert::TryInto;

/// A 16 byte with the UUID.
pub type Bytes = [u8; 16];

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

/// Universally Unique Identifier, or UUID.
///
/// This type is `repr(transparent)` and guaranteed to have the same layout
/// as `[u8; 16]`.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Uuid(Bytes);

impl Uuid {
    /// The special Nil UUID, where all bits are set to zero.
    pub fn nil() -> Self {
        Uuid(Bytes::default())
    }

    /// Create a UUID from bytes.
    pub fn from_bytes(bytes: Bytes) -> Self {
        Self(bytes)
    }

    /// Return the UUID as it's bytes.
    pub fn to_bytes(self) -> Bytes {
        self.0
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
}

#[cfg(test)]
mod tests {
    use super::*;

    const _UUID_V4: &str = "662aa7c7-7598-4d56-8bcc-a72c30f998a2";
    const RAW: [u8; 16] = [
        102, 42, 167, 199, 117, 152, 77, 86, 139, 204, 167, 44, 48, 249, 152, 162,
    ];

    #[test]
    fn info() {
        let uuid = Uuid::from_bytes(RAW);
        assert_eq!(uuid.version(), Version::Random);
        assert_eq!(uuid.variant(), Variant::Rfc4122);
    }
}
