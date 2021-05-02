//! Create and use UUID's
#![cfg_attr(not(any(test, feature = "std")), no_std)]
use bitvec::prelude::*;
use core::{convert::TryInto, fmt, fmt::Write as _, str::FromStr};
use md5::{Digest, Md5};
use rand::prelude::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use sha1::{digest::generic_array::sequence::Shorten, Sha1};

const UUID_STR_LENGTH: usize = 36;
const UUID_URN_LENGTH: usize = 45;
const UUID_URN: &str = "urn:uuid:";

/// The predefined DNS namespace, 6ba7b810-9dad-11d1-80b4-00c04fd430c8.
pub const NAMESPACE_DNS: Uuid = Uuid::from_bytes([
    107, 167, 184, 16, 157, 173, 17, 209, 128, 180, 0, 192, 79, 212, 48, 200,
]);

/// The predefined URL namespace, 6ba7b811-9dad-11d1-80b4-00c04fd430c8.
pub const NAMESPACE_URL: Uuid = Uuid::from_bytes([
    107, 167, 184, 17, 157, 173, 17, 209, 128, 180, 0, 192, 79, 212, 48, 200,
]);

/// The predefined OID namespace, 6ba7b812-9dad-11d1-80b4-00c04fd430c8.
pub const NAMESPACE_OID: Uuid = Uuid::from_bytes([
    107, 167, 184, 18, 157, 173, 17, 209, 128, 180, 0, 192, 79, 212, 48, 200,
]);

/// The predefined X500 namespace, 6ba7b814-9dad-11d1-80b4-00c04fd430c8.
pub const NAMESPACE_X500: Uuid = Uuid::from_bytes([
    107, 167, 184, 20, 157, 173, 17, 209, 128, 180, 0, 192, 79, 212, 48, 200,
]);

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

impl<'a> fmt::Write for BytesWrapper<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if (self.bytes.len() - self.offset) < s.len() {
            return Err(fmt::Error);
        }
        self.bytes[self.offset..][..s.len()].copy_from_slice(s.as_bytes());
        self.offset += s.len();
        Ok(())
    }
}

/// A CSPRNG suitable for generating UUID's.
#[derive(Debug, Clone)]
pub struct Rng(rand::rngs::StdRng);

impl Rng {
    /// Create a new Rng using getrandom.
    #[cfg(feature = "getrandom")]
    pub fn new() -> Self {
        Self(StdRng::from_rng(rand::rngs::OsRng).unwrap())
    }

    /// Create a new Rng from a provided seed.
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self(StdRng::from_seed(seed))
    }

    /// Forward to rand's fill_bytes
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest)
    }
}

#[cfg(feature = "getrandom")]
impl Default for Rng {
    #[inline]
    fn default() -> Self {
        Self::new()
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

    /// Special case for invalid UUIDs.
    Invalid,
}

/// Error parsing UUID
#[derive(Debug)]
pub struct ParseUuidError;

impl fmt::Display for ParseUuidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParseUuidError")
    }
}

#[cfg(any(test, feature = "std"))]
impl std::error::Error for ParseUuidError {}

/// Universally Unique Identifier, or UUID.
///
/// This type is `repr(transparent)` and guaranteed to have the same layout
/// as `[u8; 16]`.
///
/// UUID fields are always considered to be laid out MSB, or big-endian.
///
/// This type is also `serde(transparent)`, when serde is enabled.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(transparent))]
#[repr(transparent)]
pub struct Uuid(Bytes);

impl Uuid {
    /// Set the UUID Version.
    fn set_version(&mut self, ver: Version) {
        // The version is in the 4 highest bits, so we only need the first byte.
        // Clear the 4 highest bits.
        self.0[6] &= 0x0F;

        match ver {
            Version::Time => {
                self.0[6] |= 1u8 << 4;
            }
            Version::Dce => {
                self.0[6] |= 2u8 << 4;
            }
            Version::Md5 => {
                self.0[6] |= 3u8 << 4;
            }
            Version::Random => {
                self.0[6] |= 4u8 << 4;
            }
            Version::Sha1 => {
                self.0[6] |= 5u8 << 4;
            }
            Version::Nil => unreachable!("Can't set UUID to nil version"),
            Version::Invalid => unreachable!("Can't set UUID to invalid version"),
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
        let bits = self.0[8].view_bits_mut::<Msb0>();
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

    /// Swap the in-memory format between big-endian and mixed-endian.
    #[inline]
    fn swap_endian(mut self) -> Self {
        self.0[0..4].reverse();
        self.0[4..6].reverse();
        self.0[6..8].reverse();
        self
    }
}

impl Uuid {
    /// The special Nil UUID, where all bits are set to zero.
    #[inline]
    pub const fn nil() -> Self {
        Uuid([0; 16])
    }

    /// Create a UUID from bytes.
    #[inline]
    pub const fn from_bytes(bytes: Bytes) -> Self {
        Self(bytes)
    }

    /// Return the UUID as it's bytes.
    #[inline]
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
    #[inline]
    pub fn from_bytes_me(bytes: Bytes) -> Self {
        Self(bytes).swap_endian()
    }

    /// Return the UUID as mixed-endian bytes.
    ///
    /// See [`Uuid::from_bytes_me`] for details.
    #[inline]
    pub fn to_bytes_me(self) -> Bytes {
        self.swap_endian().to_bytes()
    }

    /// Returns true if the UUID is nil.
    #[inline]
    pub fn is_nil(self) -> bool {
        self.0 == Self::nil().0
    }

    /// The UUID Variant
    ///
    /// # Warning
    ///
    /// Many UUIDs out in the wild are incorrectly generated,
    /// so this value can't be trusted.
    #[inline]
    pub fn variant(self) -> Variant {
        let bits = &self.0[8].view_bits::<Msb0>()[..3];
        match (bits[0], bits[1], bits[2]) {
            (true, true, true) => Variant::Reserved,
            (true, true, false) => Variant::Microsoft,
            (true, false, ..) => Variant::Rfc4122,
            (false, ..) => Variant::Ncs,
        }
    }

    /// The UUID Variant
    ///
    /// # Warning
    ///
    /// Many UUIDs out in the wild are incorrectly generated,
    /// so this value can't be trusted.
    #[inline]
    pub fn version(self) -> Version {
        let bits = &self.0[6].view_bits::<Msb0>()[..4];
        match (bits[0], bits[1], bits[2], bits[3]) {
            (false, false, false, false) => Version::Nil,
            (false, false, false, true) => Version::Time,
            (false, false, true, false) => Version::Dce,
            (false, false, true, true) => Version::Md5,
            (false, true, false, false) => Version::Random,
            (false, true, false, true) => Version::Sha1,
            _ => Version::Invalid,
        }
    }

    /// Write UUID as a lowercase ASCII string into `buf`, and returns it as a
    /// string.
    ///
    /// # Panics
    ///
    /// If `buf.len()` is not == 36
    // TODO: Use arrays? As of 1.47 they have const generic impls.
    // But still have to return a `&str`..
    // Maybe `&mut [u8; 36]`? `TryFrom` is impled for that
    pub fn to_str(self, buf: &mut [u8]) -> &mut str {
        assert!(buf.len() == 36, "Uuid::to_str requires 36 bytes");
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
        let mut buf = BytesWrapper::new(buf);
        write!(
            buf,
            "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:012x}",
            time_low, time_mid, time_hi_and_version, clock_seq_hi_and_reserved, clock_seq_low, node
        )
        .expect("BUG: Couldn't write UUID");
        core::str::from_utf8_mut(buf.into_inner()).expect("BUG: Invalid UTF8")
    }

    /// Write a UUID as a lowercase ASCII string into `buf`, and return it as a
    /// string.
    ///
    /// # Panics
    ///
    /// If `buf.len()` is not == 45
    // TODO: Use arrays? As of 1.47 they have const generic impls.
    // But still have to return a `&str`..
    // Maybe `&mut [u8; 45]`? `TryFrom` is impled for that
    pub fn to_urn(self, buf: &mut [u8]) -> &mut str {
        assert!(buf.len() == 45, "Uuid::to_urn requires 45 bytes");
        buf[..UUID_URN.len()].copy_from_slice(UUID_URN.as_bytes());
        self.to_str(&mut buf[UUID_URN.len()..]);
        core::str::from_utf8_mut(buf).expect("BUG: Invalid UTF8")
    }

    /// [`Uuid::to_str`], but uppercase.
    pub fn to_str_upper(self, buf: &mut [u8]) -> &mut str {
        let s = self.to_str(buf);
        s.make_ascii_uppercase();
        s
    }

    /// [`Uuid::to_urn`], but the UUID is uppercase.
    pub fn to_urn_upper(self, buf: &mut [u8]) -> &mut str {
        let s = self.to_urn(buf);
        s[UUID_URN.len()..].make_ascii_uppercase();
        s
    }
}

impl Uuid {
    /// Parse a [`Uuid`] from a string
    ///
    /// Case insensitive and supports "urn:uuid:"
    ///
    /// # Example
    ///
    /// ```rust
    /// # use nuuid::Uuid;
    /// Uuid::parse("662aa7c7-7598-4d56-8bcc-a72c30f998a2").unwrap();
    /// Uuid::parse("662AA7C7-7598-4D56-8BCC-A72C30F998A2").unwrap();
    ///
    /// Uuid::parse("urn:uuid:662aa7c7-7598-4d56-8bcc-a72c30f998a2").unwrap();
    /// Uuid::parse("urn:uuid:662AA7C7-7598-4D56-8BCC-A72C30F998A2").unwrap();
    /// ```
    #[inline]
    pub fn parse(s: &str) -> Result<Self, ParseUuidError> {
        Uuid::from_str(s)
    }

    /// Create a new Version 4(Random) UUID.
    ///
    /// This requires the `getrandom` feature.
    ///
    /// If generating a lot of UUID's very quickly, prefer [`Uuid::new_v4_rng`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use nuuid::Uuid;
    /// let uuid = Uuid::new_v4();
    /// ```
    #[cfg(feature = "getrandom")]
    pub fn new_v4() -> Self {
        let mut uuid = Uuid::nil();
        Rng::new().fill_bytes(&mut uuid.0);
        uuid.set_variant(Variant::Rfc4122);
        uuid.set_version(Version::Random);
        uuid
    }

    /// Create a new Version 4(Random) UUID, using the provided [`Rng`]
    ///
    /// This method is useful if you need to generate a lot of UUID's very
    /// quickly, since it won't create and seed a new RNG each time.
    ///
    /// Providing a good seed is left to you, however.
    /// If a bad seed is used, the resulting UUIDs may not be
    /// sufficiently random or unique.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use nuuid::{Rng, Uuid};
    /// # let seed = [0; 32];
    /// let mut rng = Rng::from_seed(seed);
    /// for _ in 0..10 {
    ///     let uuid = Uuid::new_v4_rng(&mut rng);
    /// }
    /// ```
    pub fn new_v4_rng(rng: &mut Rng) -> Self {
        let mut uuid = Uuid::nil();
        rng.fill_bytes(&mut uuid.0);
        uuid.set_variant(Variant::Rfc4122);
        uuid.set_version(Version::Random);
        uuid
    }

    /// Create a new Version 3 UUID with the provided name and namespace.
    ///
    /// # Note
    ///
    /// Version 3 UUID's use the obsolete MD5 algorithm,
    /// [`Uuid::new_v5`] should be preferred.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use nuuid::{NAMESPACE_DNS, Uuid};
    /// let uuid = Uuid::new_v3(NAMESPACE_DNS, b"example.com");
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// # use nuuid::{NAMESPACE_DNS, Uuid};
    /// let uuid = Uuid::new_v5(NAMESPACE_DNS, b"example.com");
    /// ```
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

/// See [`Uuid::parse`] for details.
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

/// Display the [`Uuid`] in uppercase hex.
///
/// # Example
///
/// ```rust
/// # use nuuid::Uuid;
/// let uuid = Uuid::parse("662aa7c7-7598-4d56-8bcc-a72c30f998a2").unwrap();
/// assert_eq!(format!("{}", uuid), "662AA7C7-7598-4D56-8BCC-A72C30F998A2");
/// ```
impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:X}", self)
    }
}

impl fmt::Debug for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Uuid({:X})", self)
    }
}

/// Display the [`Uuid`] in lowercase
///
/// The alternate(`#`) flag can be used to get a URN.
///
/// # Example
///
/// ```rust
/// # use nuuid::Uuid;
/// let uuid = Uuid::parse("662aa7c7-7598-4d56-8bcc-a72c30f998a2").unwrap();
/// assert_eq!(format!("{:x}", uuid), "662aa7c7-7598-4d56-8bcc-a72c30f998a2");
/// assert_eq!(format!("{:#x}", uuid), "urn:uuid:662aa7c7-7598-4d56-8bcc-a72c30f998a2");
/// ```
impl fmt::LowerHex for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}", UUID_URN)?;
        }
        let mut buf = [0; 36];
        let s = self.to_str(&mut buf);
        write!(f, "{}", s)
    }
}

/// Display the [`Uuid`] in uppercase
///
/// The alternate(`#`) flag can be used to get a URN.
///
/// # Example
///
/// ```rust
/// # use nuuid::Uuid;
/// let uuid = Uuid::parse("662aa7c7-7598-4d56-8bcc-a72c30f998a2").unwrap();
/// assert_eq!(format!("{:X}", uuid), "662AA7C7-7598-4D56-8BCC-A72C30F998A2");
/// assert_eq!(format!("{:#X}", uuid), "urn:uuid:662AA7C7-7598-4D56-8BCC-A72C30F998A2");
/// ```
impl fmt::UpperHex for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}", UUID_URN)?;
        }
        let mut buf = [0; 36];
        write!(f, "{}", self.to_str_upper(&mut buf))
    }
}

impl AsRef<[u8]> for Uuid {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8; 16]> for Uuid {
    #[inline]
    fn as_ref(&self) -> &[u8; 16] {
        &self.0
    }
}

// NOTE: Should this impl exist?
impl From<[u8; 16]> for Uuid {
    #[inline]
    fn from(b: [u8; 16]) -> Self {
        Uuid::from_bytes(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UUID_NIL: &str = "00000000-0000-0000-0000-000000000000";
    const UUID_V4: &str = "662aa7c7-7598-4d56-8bcc-a72c30f998a2";
    const UUID_V4_URN: &str = "urn:uuid:662aa7c7-7598-4d56-8bcc-a72c30f998a2";
    const UUID_V4_URN_UPPER: &str = "urn:uuid:662AA7C7-7598-4D56-8BCC-A72C30F998A2";
    const RAW: [u8; 16] = [
        102, 42, 167, 199, 117, 152, 77, 86, 139, 204, 167, 44, 48, 249, 152, 162,
    ];

    fn name(fun: fn(Uuid, &[u8]) -> Uuid, ver: Version) {
        let namespace = Uuid::new_v4();
        let namespace2 = Uuid::new_v4();
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
        assert_eq!(
            &uuid.to_str(&mut buf[..36])[..],
            UUID_V4,
            "UUID strings didn't match"
        );
        assert_eq!(
            uuid.to_urn(&mut buf),
            UUID_V4_URN,
            "UUID URN strings didn't match"
        );
        assert_eq!(
            uuid.to_urn_upper(&mut buf),
            UUID_V4_URN_UPPER,
            "UUID URN upper strings didn't match"
        );
        assert_eq!(
            format!("{:#x}", uuid),
            UUID_V4_URN,
            "UUID URN Display didn't match"
        );
        assert_eq!(format!("{:x}", uuid), UUID_V4, "UUID Display didn't match");
        assert_eq!(
            format!("{}", uuid),
            UUID_V4.to_ascii_uppercase(),
            "UUID Display didn't match"
        );
        assert_eq!(
            format!("{}", Uuid::nil()),
            UUID_NIL,
            "Nil UUID Display didn't work!"
        );
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
