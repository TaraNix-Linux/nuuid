//! Create and use UUID's
#![cfg_attr(not(test), no_std)]
use bitvec::prelude::*;
use core::{
    convert::TryInto,
    fmt::{Error, Result as FmtResult, Write},
    str::FromStr,
};
use md5::{Digest, Md5};
use rand::prelude::*;
use sha1::{digest::generic_array::sequence::Shorten, Sha1};

const UUID_STR_LENGTH: usize = 36;
const UUID_URN_LENGTH: usize = 45;
const UUID_URN: &str = "urn:uuid:";

/// A 16 byte with the UUID.
pub type Bytes = [u8; 16];

/// Used to write out UUID's to a user-provided buffer.
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
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(transparent)]
// TODO: Better Debug, Display. Test Eq/Ord. Examples
pub struct Uuid(Bytes);

impl Uuid {
    /// Set the UUID Version.
    fn set_version(&mut self, ver: Version) {
        // The version is in the 4 highest bits, so we only need the first byte.
        let bits = self.0[6].bits_mut::<Msb0>();
        let bits = &mut bits[..4];
        bits.set_all(false);
        match ver {
            Version::Time => bits.store_be(1u8),
            Version::Dce => bits.store_be(2u8),
            Version::Md5 => bits.store_be(3u8),
            Version::Random => bits.store_be(4u8),
            Version::Sha1 => bits.store_be(5u8),
            Version::Nil => unreachable!("Can't set UUID to nil version"),
        }
    }

    /// Set the UUID Variant, only touching bits as specified.
    ///
    /// The version field has several unspecified bits, which this method
    /// leaves alone. Legacy UUID's can thus be modified losslessly.
    ///
    /// When creating UUID's, these unspecified bits should always be zero by
    /// default anyway.
    fn set_variant(&mut self, ver: Variant) {
        // The variant is, variably, in the 3 highest bits.
        let bits = self.0[8].bits_mut::<Msb0>();
        let bits = &mut bits[..3];
        match ver {
            Variant::Ncs => bits.set(0, true),
            Variant::Rfc4122 => {
                bits.set(0, true);
                bits.set(1, false);
            }
            Variant::Microsoft => {
                bits.set(0, true);
                bits.set(1, true);
                bits.set(2, false);
            }
            Variant::Reserved => bits.set_all(true),
        }
    }
}

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
            v => panic!("Invalid version: {:?}", v),
        }
    }

    /// Write UUID as the ASCII string into `buf`, and returns it as a string.
    ///
    /// # Panics
    ///
    /// If `buf.len()` is not >= 36
    // TODO: Use array when const stuff improves?
    // Right now try_into only exists for up to 32, so requiring an
    // array here would be inconvenient in practice.
    pub fn to_string(self, buf: &mut [u8]) -> &str {
        assert!(buf.len() >= 36, "Buf too small for UUID");
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
        core::str::from_utf8(buf.into_inner()).expect("BUG: Invalid UTF8")
    }

    /// Write a UUID as a ASCII string into `buf`, and return it as a string.
    ///
    /// # Panics
    ///
    /// If `buf.len()` is not >= 45
    // TODO: Use array when const stuff improves?
    // Right now try_into only exists for up to 32, so requiring an
    // array here would be inconvenient in practice.
    pub fn to_urn(self, buf: &mut [u8]) -> &str {
        assert!(buf.len() >= 45, "Buf too small for UUID");
        buf[..9].copy_from_slice(b"urn:uuid:");
        self.to_string(&mut buf[9..]);
        core::str::from_utf8(buf).expect("BUG: Invalid UTF8")
    }
}

impl Uuid {
    /// Create a new Version 4(Random) UUID.
    ///
    /// This requires the `getrandom` feature.
    #[cfg(feature = "getrandom")]
    pub fn new_v4() -> Self {
        let mut seed = [0; 32];
        StdRng::from_entropy().fill_bytes(&mut seed);
        Self::new_v4_seed(seed)
    }

    /// Create a new Version 4(Random) UUID,
    /// using the provided seed.
    ///
    /// `seed` is used to initialize a suitable CSPRNG
    pub fn new_v4_seed(seed: [u8; 32]) -> Self {
        let mut bytes = [0; 16];
        StdRng::from_seed(seed).fill_bytes(&mut bytes);
        let mut uuid = Uuid::from_bytes(bytes);
        uuid.set_variant(Variant::Rfc4122);
        uuid.set_version(Version::Random);
        uuid
    }

    /// Create a new Version 3 UUID with the provided name and namespace.
    ///
    /// # Note
    ///
    /// Version 3 UUID's use the obsolete MD5 algorithm.
    #[deprecated = "Version 3 UUID's use MD5. Prefer Uuid::new_v5, which uses SHA-1."]
    pub fn new_v3(namespace: Uuid, name: &[u8]) -> Self {
        let mut hasher = Md5::new();
        hasher.update(namespace.to_bytes());
        hasher.update(name);
        let mut uuid = Uuid::from_bytes(hasher.finalize().into());
        uuid.set_version(Version::Md5);
        uuid.set_variant(Variant::Rfc4122);
        uuid
    }

    /// Create a new Version 5 UUID with the provided name and namespace.
    pub fn new_v5(namespace: Uuid, name: &[u8]) -> Self {
        let mut hasher = Sha1::new();
        hasher.update(namespace.to_bytes());
        hasher.update(name);
        let mut uuid = Uuid::from_bytes(
            hasher
                .finalize()
                .pop_back()
                .0
                .pop_back()
                .0
                .pop_back()
                .0
                .pop_back()
                .0
                .into(),
        );
        uuid.set_version(Version::Sha1);
        uuid.set_variant(Variant::Rfc4122);
        uuid
    }
}

/// Parse a [`Uuid`] from a string
///
/// Case insensitive and supports "urn:uuid:"
impl FromStr for Uuid {
    type Err = ParseUuidError;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        if s.len() == UUID_URN_LENGTH {
            s = &s[UUID_URN.len()..];
        }
        if s.len() != UUID_STR_LENGTH {
            return Err(ParseUuidError);
        }
        let mut raw = [0; 16];
        let mut buf: &mut [u8] = &mut raw;
        for data in s.split('-') {
            match data.len() {
                8 => {
                    buf[..4].copy_from_slice(
                        &u32::from_str_radix(data, 16)
                            .or(Err(ParseUuidError))?
                            .to_be_bytes(),
                    );
                    buf = &mut buf[4..];
                }
                4 => {
                    buf[..2].copy_from_slice(
                        &u16::from_str_radix(data, 16)
                            .or(Err(ParseUuidError))?
                            .to_be_bytes(),
                    );
                    buf = &mut buf[2..];
                }
                12 => {
                    buf[..6].copy_from_slice(
                        &u64::from_str_radix(data, 16)
                            .or(Err(ParseUuidError))?
                            .to_be_bytes()[2..],
                    );
                }
                _ => return Err(ParseUuidError),
            }
        }
        Ok(Uuid::from_bytes(raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UUID_V4: &str = "662aa7c7-7598-4d56-8bcc-a72c30f998a2";
    const UUID_V4_URN: &str = "urn:uuid:662aa7c7-7598-4d56-8bcc-a72c30f998a2";
    const RAW: [u8; 16] = [
        102, 42, 167, 199, 117, 152, 77, 86, 139, 204, 167, 44, 48, 249, 152, 162,
    ];

    fn name(fun: fn(Uuid, &[u8]) -> Uuid, ver: Version) {
        let namespace = Uuid::from_bytes(RAW);
        let namespace2 = Uuid::new_v4_seed([0; 32]);
        //
        let uuid1 = fun(namespace, b"test");
        // Maybe don't?
        std::thread::sleep(std::time::Duration::from_millis(500));
        let uuid2 = fun(namespace, b"test");
        assert_eq!(
            uuid1, uuid2,
            "V3 UUID's from different times with the same name/namespace must be equal"
        );

        let uuid = fun(namespace, b"Cat");
        assert_ne!(
            uuid, uuid2,
            "UUID's with two different names in the same namespace must NOT be equal"
        );

        let uuid = fun(namespace2, b"test");
        assert_ne!(
            uuid, uuid2,
            "UUID's with the same names in a different namespace must NOT be equal"
        );

        assert_eq!(uuid.version(), ver);
        assert_eq!(uuid.variant(), Variant::Rfc4122);
    }

    #[test]
    #[allow(deprecated)]
    fn md5() {
        name(
            |namespace, name| Uuid::new_v3(namespace, name),
            Version::Md5,
        )
    }

    #[test]
    fn sha1() {
        name(
            |namespace, name| Uuid::new_v5(namespace, name),
            Version::Sha1,
        )
    }

    #[test]
    fn parse_string() {
        let uuid = Uuid::from_str(UUID_V4).unwrap();
        assert_eq!(RAW, uuid.to_bytes(), "Parsed UUID bytes don't match");
        let uuid = Uuid::from_str(UUID_V4_URN).unwrap();
        assert_eq!(RAW, uuid.to_bytes(), "Parsed UUID bytes don't match");
    }

    #[test]
    fn string() {
        let uuid = Uuid::from_bytes(RAW);
        let mut buf = [0; 45];
        let s = uuid.to_string(&mut buf);
        println!("UUID: `{}`", s);
        assert_eq!(&s[..36], UUID_V4, "UUID strings didn't match");
        let s = uuid.to_urn(&mut buf);
        assert_eq!(s, UUID_V4_URN, "UUID URN strings didn't match");
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
        #[cfg(feature = "getrandom")]
        {
            let uuid = Uuid::new_v4();
            assert_eq!(uuid.version(), Version::Random);
            assert_eq!(uuid.variant(), Variant::Rfc4122);
        }
    }
}
