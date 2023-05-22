//! Create and use UUID's
#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
use core::{
    convert::TryInto,
    fmt,
    str::{from_utf8_unchecked_mut, FromStr},
};

use hex_simd::{decode_inplace, AsciiCase::Lower, Out};
use md5::{Digest, Md5};
#[cfg(feature = "getrandom")]
use rand_chacha::rand_core::OsRng;
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaChaRng,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use sha1::Sha1;

const UUID_STR_LENGTH: usize = 36;
const UUID_URN_LENGTH: usize = 45;
const UUID_BRACED_LENGTH: usize = 38;
const UUID_SIMPLE_LENGTH: usize = 32;
const UUID_URN: &str = "urn:uuid:";
const UUID_URN_PREFIX: usize = UUID_URN.len();

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

/// A CSPRNG suitable for generating UUID's.
#[derive(Debug, Clone)]
pub struct Rng(ChaChaRng);

impl Rng {
    /// Create a new Rng using getrandom.
    #[cfg(feature = "getrandom")]
    #[cfg_attr(docsrs, doc(cfg(feature = "getrandom")))]
    #[inline]
    pub fn new() -> Self {
        Self(ChaChaRng::from_rng(OsRng).unwrap())
    }

    /// Create a new Rng from a provided seed.
    #[inline]
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self(ChaChaRng::from_seed(seed))
    }

    /// Forward to rand's fill_bytes
    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "getrandom")))]
#[cfg(feature = "getrandom")]
impl Default for Rng {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// UUID Variants
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[non_exhaustive]
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

impl fmt::Display for Variant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Variant::Ncs => write!(f, "Ncs"),
            Variant::Rfc4122 => write!(f, "Rfc4122"),
            Variant::Microsoft => write!(f, "Microsoft"),
            Variant::Reserved => write!(f, "Reserved"),
        }
    }
}

/// UUID Version
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[non_exhaustive]
pub enum Version {
    /// Special case for the nil UUID.
    Nil = 0,

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

    /// Version 6, re-ordered version of [`Version::time`] for DB locality.
    #[cfg(feature = "experimental_uuid")]
    #[cfg_attr(docsrs, doc(cfg(feature = "experimental_uuid")))]
    Database,

    /// Version 7, unix time based.
    #[cfg(feature = "experimental_uuid")]
    #[cfg_attr(docsrs, doc(cfg(feature = "experimental_uuid")))]
    UnixTime,

    /// Version 8, experimental or vendor specific format
    #[cfg(feature = "experimental_uuid")]
    #[cfg_attr(docsrs, doc(cfg(feature = "experimental_uuid")))]
    Vendor,

    /// Reserved versions. Currently, versions 9-15 are reserved.
    Reserved,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Version::Nil => write!(f, "Nil"),
            Version::Time => write!(f, "Time"),
            Version::Dce => write!(f, "Dce"),
            Version::Md5 => write!(f, "Md5"),
            Version::Random => write!(f, "Random"),
            Version::Sha1 => write!(f, "Sha1"),

            #[cfg(feature = "experimental_uuid")]
            Version::Database => write!(f, "Database"),
            #[cfg(feature = "experimental_uuid")]
            Version::UnixTime => write!(f, "UnixTime"),
            #[cfg(feature = "experimental_uuid")]
            Version::Vendor => write!(f, "Vendor"),

            Version::Reserved => write!(f, "Reserved"),
        }
    }
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
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for ParseUuidError {}

/// Universally Unique Identifier, or UUID.
///
/// This type is `repr(transparent)` and guaranteed to have the same layout
/// as `[u8; 16]`.
///
/// The various methods on `Uuid` assume each field
/// is laid out Most Significant Byte First/MSB/Big-Endian/Network Endian.
///
/// This type is also `serde(transparent)`, when serde is enabled.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(transparent))]
#[repr(transparent)]
pub struct Uuid(Bytes);

impl Uuid {
    /// Set the UUID Version.
    #[inline]
    fn set_version(&mut self, ver: Version) {
        // `Version` enum matches version layout
        self.0[6] = (self.0[6] & 0xF) | ((ver as u8) << 4);
    }

    /// Set the UUID Variant, only touching bits as specified.
    ///
    /// The version field has several unspecified bits, which this method
    /// leaves alone. Legacy UUID's can thus be modified losslessly.
    ///
    /// When creating UUID's, these unspecified bits should always be zero by
    /// default anyway.
    #[inline]
    fn set_variant(&mut self, ver: Variant) {
        let byte = self.0[8];
        self.0[8] = match ver {
            // 0xx
            Variant::Ncs => byte & 0x7F,
            // 10x
            Variant::Rfc4122 => (byte & 0x3F) | 0x80,
            // 110
            Variant::Microsoft => (byte & 0x1F) | 0xC0,
            // 111
            Variant::Reserved => byte | 0xE0,
        }
    }

    /// Swap the in-memory format between big-endian and mixed-endian.
    #[inline]
    const fn swap_endian(mut self) -> Self {
        // TODO: Const slice reverse pls. or const mem::swap.
        let (a1, a2, a3, a4) = (self.0[0], self.0[1], self.0[2], self.0[3]);
        self.0[0] = a4;
        self.0[1] = a3;
        self.0[2] = a2;
        self.0[3] = a1;

        let (a1, a2) = (self.0[4], self.0[5]);
        self.0[4] = a2;
        self.0[5] = a1;

        let (a1, a2) = (self.0[6], self.0[7]);
        self.0[6] = a2;
        self.0[7] = a1;

        self
    }
}

impl Uuid {
    /// The special Nil UUID, where all bits are set to zero.
    #[inline]
    pub const fn nil() -> Self {
        Uuid([0; 16])
    }

    /// The special Max UUID, where all bits are set to one.
    #[inline]
    #[cfg(feature = "experimental_uuid")]
    #[cfg_attr(docsrs, doc(cfg(feature = "experimental_uuid")))]
    pub const fn max() -> Self {
        Uuid([1; 16])
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
    /// The resulting UUID will be stored in-memory as big-endian.
    ///
    /// This will primarily come up when interacting with Microsoft GUIDs/UUIDs
    ///
    /// The following fields are expected to be little-endian instead of
    /// big-endian:
    ///
    /// - `time_low`
    /// - `time_mid`
    /// - `time_hi_and_version`
    ///
    /// Other fields are left unchanged
    #[inline]
    pub const fn from_bytes_me(bytes: Bytes) -> Self {
        Self(bytes).swap_endian()
    }

    /// Return the UUID as mixed-endian bytes.
    ///
    /// See [`Uuid::from_bytes_me`] for details.
    #[inline]
    pub const fn to_bytes_me(self) -> Bytes {
        self.swap_endian().to_bytes()
    }

    /// Returns true if the UUID is nil.
    #[inline]
    pub const fn is_nil(self) -> bool {
        // Done this way for const reasons
        // nil is nil regardless of byte order
        u128::from_ne_bytes(self.0) == 0
    }

    /// The UUID Variant
    ///
    /// # Warning
    ///
    /// Many UUIDs out in the wild are incorrectly generated,
    /// so this value can't be relied upon.
    #[inline]
    pub const fn variant(self) -> Variant {
        // Check the highest 3 bits, skipping the 5 bits of the time/clock sequence
        let byte = (self.0[8] >> 5) & 0b111;

        // Done this way for exhaustiveness checking.
        match (
            // Msb0
            byte >> 2 & 1 == 1,
            // Msb1
            byte >> 1 & 1 == 1,
            // Msb2
            byte & 1 == 1,
        ) {
            (false, ..) => Variant::Ncs,
            (true, false, ..) => Variant::Rfc4122,
            (true, true, false) => Variant::Microsoft,
            (true, true, true) => Variant::Reserved,
        }
    }

    /// The UUID Version
    ///
    /// For "incorrect" or non-RFC UUIDs, this value may be nonsensical.
    ///
    /// If the version bits do not match any known version,
    /// [`Version::Reserved`] is returned instead.
    /// As new versions are recognized, this may change.
    ///
    /// # Warning
    ///
    /// Many UUIDs out in the wild are incorrectly generated,
    /// so this value can't be relied upon.
    #[inline]
    pub const fn version(self) -> Version {
        // Check the highest 4 bits
        match (
            self.0[6] >> 7 & 1 == 1,
            self.0[6] >> 6 & 1 == 1,
            self.0[6] >> 5 & 1 == 1,
            self.0[6] >> 4 & 1 == 1,
        ) {
            (false, false, false, false) => Version::Nil,
            (false, false, false, true) => Version::Time,
            (false, false, true, false) => Version::Dce,
            (false, false, true, true) => Version::Md5,
            (false, true, false, false) => Version::Random,
            (false, true, false, true) => Version::Sha1,

            #[cfg(feature = "experimental_uuid")]
            (false, true, true, false) => Version::Database,

            #[cfg(feature = "experimental_uuid")]
            (false, true, true, true) => Version::UnixTime,

            #[cfg(feature = "experimental_uuid")]
            (true, false, false, false) => Version::Vendor,

            _ => Version::Reserved,
        }
    }

    /// The 60-bit UUID timestamp
    ///
    /// This value will only make sense for [`Version::Time`] or
    /// [`Version::Database`] UUIDs
    ///
    /// The value of this will depend on [`Uuid::version`]
    #[inline]
    pub const fn timestamp(self) -> u64 {
        match self.version() {
            #[cfg(feature = "experimental_uuid")]
            Version::Database => u64::from_be_bytes([
                // Clear version bits
                self.0[6] & 0xF,
                self.0[7],
                self.0[4],
                self.0[5],
                self.0[0],
                self.0[1],
                self.0[2],
                self.0[3],
            ]),
            // #[cfg(feature = "experimental_uuid")]
            // Version::UnixTime => todo!(),
            _ => u64::from_be_bytes([
                // Clear version bits
                self.0[6] & 0xF,
                self.0[7],
                self.0[4],
                self.0[5],
                self.0[0],
                self.0[1],
                self.0[2],
                self.0[3],
            ]),
        }
    }

    /// The 14-bit UUID clock sequence
    ///
    /// This value will only make sense for [`Version::Time`] or
    /// [`Version::Database`] UUIDs
    ///
    /// The value of this will depend on [`Uuid::version`]
    #[inline]
    pub const fn clock_sequence(self) -> u16 {
        u16::from_be_bytes([
            // Clear variant bits
            // Only need to clear two because this only makes sense for RFC UUIDs
            self.0[8] & 0x3F,
            self.0[9],
        ])
    }

    /// The 48-bit UUID Node ID
    #[inline]
    pub const fn node(self) -> [u8; 6] {
        [
            self.0[10], self.0[11], self.0[12], self.0[13], self.0[14], self.0[15],
        ]
    }

    /// Write UUID as a lowercase ASCII string into `buf`, and returns it as a
    /// string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use nuuid::Uuid;
    /// const EXAMPLE_UUID: &str = "662aa7c7-7598-4d56-8bcc-a72c30f998a2";
    /// let uuid = Uuid::parse(EXAMPLE_UUID)?;
    ///
    /// let mut buf = [0u8; 36];
    /// let string = uuid.to_str(&mut buf);
    /// assert_eq!(string, EXAMPLE_UUID);
    /// # Ok(()) }
    /// ```
    ///
    /// With an array
    ///
    /// ```rust
    /// # use nuuid::Uuid;
    /// let uuid = Uuid::new_v4();
    /// let mut buf = [0u8; 36];
    /// let string = uuid.to_str(&mut buf);
    /// ```
    ///
    /// With a slice
    ///
    /// ```rust
    /// # use nuuid::Uuid;
    /// # use std::convert::TryInto;
    /// let uuid = Uuid::new_v4();
    /// let mut data = [0u8; 50];
    /// let string = uuid.to_str((&mut data[..36]).try_into().unwrap());
    /// ```
    ///
    /// With a slice, incorrectly.
    ///
    /// The problem here is that the slices length is unconstrained, and could
    /// be more or less than 36.
    ///
    /// ```rust,should_panic
    /// # use nuuid::Uuid;
    /// # use std::convert::TryInto;
    /// let uuid = Uuid::new_v4();
    /// let mut data = [0u8; 50];
    /// let string = uuid.to_str((&mut data[..]).try_into().unwrap());
    /// ```
    pub fn to_str(self, buf: &mut [u8; 36]) -> &mut str {
        // Add hyphens
        buf[8] = b'-';
        buf[13] = b'-';
        buf[18] = b'-';
        buf[23] = b'-';

        let bytes = self.to_bytes();

        let time_low = &bytes[..4];
        let time_mid = &bytes[4..6];
        let time_hi_and_version = &bytes[6..8];
        let clock_seq_hi_and_reserved = &bytes[8..9];
        let clock_seq_low = &bytes[9..10];
        let node = &bytes[10..];

        // Encode each field of the UUID
        let _ = hex_simd::encode(time_low, Out::from_slice(&mut buf[..8]), Lower);
        let _ = hex_simd::encode(time_mid, Out::from_slice(&mut buf[9..13]), Lower);
        let _ = hex_simd::encode(
            time_hi_and_version,
            Out::from_slice(&mut buf[14..18]),
            Lower,
        );
        let _ = hex_simd::encode(
            clock_seq_hi_and_reserved,
            Out::from_slice(&mut buf[19..21]),
            Lower,
        );
        let _ = hex_simd::encode(clock_seq_low, Out::from_slice(&mut buf[21..23]), Lower);
        let _ = hex_simd::encode(node, Out::from_slice(&mut buf[24..]), Lower);

        debug_assert!(buf.is_ascii(), "BUG: Invalid ASCII in nuuid::Uuid::to_str");
        // This is consistently faster than using the safe checked variant.
        // Safety: Fully initialized with ASCII hex
        unsafe { from_utf8_unchecked_mut(buf) }
    }

    /// Write a UUID as a lowercase ASCII string into `buf`, and return it as a
    /// string.
    ///
    /// For usage examples see [`Uuid::to_str`].
    #[inline]
    pub fn to_urn(self, buf: &mut [u8; 45]) -> &mut str {
        buf[..UUID_URN_PREFIX].copy_from_slice(UUID_URN.as_bytes());
        self.to_str((&mut buf[UUID_URN_PREFIX..]).try_into().unwrap());
        core::str::from_utf8_mut(buf).expect("BUG: Invalid UTF8")
    }

    /// [`Uuid::to_str`], but uppercase.
    #[inline]
    pub fn to_str_upper(self, buf: &mut [u8; 36]) -> &mut str {
        let s = self.to_str(buf);
        s.make_ascii_uppercase();
        s
    }

    /// [`Uuid::to_urn`], but the UUID is uppercase.
    #[inline]
    pub fn to_urn_upper(self, buf: &mut [u8; 45]) -> &mut str {
        let s = self.to_urn(buf);
        s[UUID_URN_PREFIX..].make_ascii_uppercase();
        s
    }
}

impl Uuid {
    /// Parse a [`Uuid`] from a string
    ///
    /// This method is case insensitive and supports the following formats:
    ///
    /// - `urn:uuid:` `urn:uuid:662aa7c7-7598-4d56-8bcc-a72c30f998a2`
    /// - "Braced" `{662aa7c7-7598-4d56-8bcc-a72c30f998a2}`
    /// - "Hyphenate" `662aa7c7-7598-4d56-8bcc-a72c30f998a2`
    /// - "Simple" `662aa7c775984d568bcca72c30f998a2`
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
    ///
    /// Uuid::parse("662aa7c775984d568bcca72c30f998a2").unwrap();
    /// Uuid::parse("662AA7C775984D568BCCA72C30F998A2").unwrap();
    ///
    /// Uuid::parse("{662aa7c7-7598-4d56-8bcc-a72c30f998a2}").unwrap();
    /// Uuid::parse("{662AA7C7-7598-4D56-8BCC-A72C30F998A2}").unwrap();
    /// ```
    pub fn parse(s: &str) -> Result<Self, ParseUuidError> {
        // Error if input is not ASCII
        if !s.is_ascii() {
            return Err(ParseUuidError);
        }

        let s = match s.len() {
            UUID_URN_LENGTH => &s[UUID_URN_PREFIX..],
            UUID_BRACED_LENGTH => &s[1..s.len() - 1],
            UUID_STR_LENGTH => s,
            UUID_SIMPLE_LENGTH => {
                return Ok(Uuid::from_bytes(
                    u128::from_str_radix(s, 16)
                        .map_err(|_| ParseUuidError)?
                        .to_be_bytes(),
                ));
            }
            _ => return Err(ParseUuidError),
        };
        let s = s.as_bytes();

        let mut raw = [0; UUID_SIMPLE_LENGTH];
        // "00000000-0000-0000-0000-000000000000"
        //          9    14   19   24
        // - 1
        // "00000000000000000000000000000000"
        //          9   13  17  21
        // - 1

        // Copy the UUID to `raw`, but without the hyphens, so we can
        // decode it in-place.
        // Benchmarking showed this was much faster than decoding directly
        // to raw in-between the hyphens.

        // Node data
        raw[20..].copy_from_slice(&s[24..]);

        // High bits of the clock, variant, and low bits of the clock
        raw[16..20].copy_from_slice(&s[19..23]);

        // High bits of the timestamp, and version
        raw[12..16].copy_from_slice(&s[14..18]);

        // Middle bits of the timestamp
        raw[8..12].copy_from_slice(&s[9..13]);

        // Low bits of the timestamp
        raw[..8].copy_from_slice(&s[..8]);

        let x = decode_inplace(&mut raw).map_err(|_| ParseUuidError)?;
        Ok(Uuid::from_bytes(x.try_into().map_err(|_| ParseUuidError)?))
    }

    /// Parse a [`Uuid`] from a string that is in mixed-endian
    ///
    /// This method is bad and should never be needed, but there are UUIDs in
    /// the wild that do this.
    ///
    /// These UUIDs are being displayed wrong, but you still need to parse them
    /// correctly.
    ///
    /// See [`Uuid::from_bytes_me`] for details.
    pub fn parse_me(s: &str) -> Result<Self, ParseUuidError> {
        Uuid::from_str(s).map(Uuid::swap_endian)
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
    #[cfg_attr(docsrs, doc(cfg(feature = "getrandom")))]
    #[inline]
    pub fn new_v4() -> Self {
        let mut uuid = Uuid::nil();
        OsRng.fill_bytes(&mut uuid.0);
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
    #[inline]
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
    #[inline]
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
    #[inline]
    pub fn new_v5(namespace: Uuid, name: &[u8]) -> Self {
        let mut hasher = Sha1::new();
        hasher.update(namespace.to_bytes());
        hasher.update(name);
        let mut uuid = Uuid::from_bytes(hasher.finalize()[..16].try_into().unwrap());
        uuid.set_version(Version::Sha1);
        uuid.set_variant(Variant::Rfc4122);
        uuid
    }

    /// Create a new Version 1 UUID using the provided 60-bit timestamp,
    /// 14-bit counter, and node.
    ///
    /// The 4 high bits of `timestamp` are ignored
    ///
    /// The 2 high bits of `counter` are ignored
    ///
    /// # Example
    ///
    /// ```rust
    /// # use nuuid::{NAMESPACE_DNS, Uuid};
    /// # let (TIMESTAMP, RANDOM, RANDOM_OR_MAC) = (0, 0, [0; 6]);
    /// let uuid = Uuid::new_v1(TIMESTAMP, RANDOM, RANDOM_OR_MAC);
    /// ```
    #[inline]
    pub fn new_v1(timestamp: u64, counter: u16, node: [u8; 6]) -> Self {
        let timestamp = timestamp.to_be_bytes();
        let counter = counter.to_be_bytes();
        Uuid::from_bytes([
            // time_low
            timestamp[4],
            timestamp[5],
            timestamp[6],
            timestamp[7],
            // time_mid
            timestamp[2],
            timestamp[3],
            // time_hi Version, ignore highest 4 bits, skip `set_version` and set the version
            (timestamp[0] & 0xF) | (1u8 << 4),
            timestamp[1],
            // clock_seq_hi Variant, skip `set_variant` and set the variant
            (counter[0] & 0x3F) | 0x80,
            counter[1],
            // Node
            node[0],
            node[1],
            node[2],
            node[3],
            node[4],
            node[5],
        ])
    }

    /// Create a new Version 6 UUID
    ///
    /// This is identical to Version 1 UUIDs (see [`Uuid::new_v1`]),
    /// except that the timestamp fields are re-ordered
    ///
    /// # Example
    ///
    /// ```rust
    /// # use nuuid::{NAMESPACE_DNS, Uuid};
    /// # let (TIMESTAMP, RANDOM, PSEUDO) = (0, 0, [0; 6]);
    /// let uuid = Uuid::new_v6(TIMESTAMP, RANDOM, PSEUDO);
    /// ```
    #[inline]
    #[cfg(feature = "experimental_uuid")]
    #[cfg_attr(docsrs, doc(cfg(feature = "experimental_uuid")))]
    pub fn new_v6(timestamp: u64, counter: u16, node: [u8; 6]) -> Self {
        // Truncate the highest 4 bits
        // https://www.ietf.org/archive/id/draft-peabody-dispatch-new-uuid-format-04.html#section-6.1-2.14
        let timestamp = (timestamp << 4).to_be_bytes();
        let counter = counter.to_be_bytes();

        Uuid::from_bytes([
            // time_high
            timestamp[0],
            timestamp[1],
            timestamp[2],
            timestamp[3],
            // time_mid
            timestamp[4],
            timestamp[5],
            // time_low Version, shift 4 bits, skip `set_version` and set the version
            (timestamp[6] >> 4) | (6u8 << 4),
            timestamp[7],
            // clock_seq_hi Variant, skip `set_variant` and set the variant
            (counter[0] & 0x3F) | 0x80,
            counter[1],
            // Node
            node[0],
            node[1],
            node[2],
            node[3],
            node[4],
            node[5],
        ])
    }

    /// Create a new Version 7 UUID
    ///
    /// This is similar to Version 1 and 6 UUIDs, but uses the UNIX epoch
    /// timestamp source.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use nuuid::{NAMESPACE_DNS, Uuid};
    /// # let (TIMESTAMP, RAND_A, RAND_B) = (0, 0, 0);
    /// let uuid = Uuid::new_v7(TIMESTAMP, RAND_A, RAND_B);
    /// ```
    #[inline]
    #[cfg(feature = "experimental_uuid")]
    #[cfg_attr(docsrs, doc(cfg(feature = "experimental_uuid")))]
    pub fn new_v7(timestamp: u64, rand_a: u16, rand_b: u64) -> Self {
        // Truncate the highest 16 bits
        // https://www.ietf.org/archive/id/draft-peabody-dispatch-new-uuid-format-04.html#section-6.1-2.14
        let timestamp = (timestamp << 16).to_be_bytes();
        let rand_a = rand_a.to_be_bytes();
        let rand_b = rand_b.to_be_bytes();

        Uuid::from_bytes([
            // unix_ts_ms
            timestamp[0],
            timestamp[1],
            timestamp[2],
            timestamp[3],
            timestamp[4],
            timestamp[5],
            // rand_a Version, ignore highest 4 bits, skip `set_version` and set the version
            (rand_a[0] & 0xF) | (7u8 << 4),
            rand_a[1],
            // rand_b, Variant, skip `set_variant` and set the variant
            (rand_b[0] & 0x3F) | 0x80,
            rand_b[1],
            rand_b[2],
            rand_b[3],
            rand_b[4],
            rand_b[5],
            rand_b[6],
            rand_b[7],
        ])
    }

    /// Create a new Version 8 UUID
    ///
    /// This will set the version and variant bits as needed,
    /// and the input will otherwise be unchanged.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use nuuid::Uuid;
    /// let uuid = Uuid::new_v8(*b"I Am 16 bytes!!!");
    /// ```
    #[inline]
    #[cfg(feature = "experimental_uuid")]
    #[cfg_attr(docsrs, doc(cfg(feature = "experimental_uuid")))]
    pub fn new_v8(bytes: Bytes) -> Self {
        let mut uuid = Uuid::from_bytes(bytes);
        uuid.set_variant(Variant::Rfc4122);
        uuid.set_version(Version::Vendor);
        uuid
    }
}

/// See [`Uuid::parse`] for details.
impl FromStr for Uuid {
    type Err = ParseUuidError;

    /// See [`Uuid::parse`] for details.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse(s)
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

/// Display the [`Uuid`] debug representation
///
/// The alternate(`#`) flag can be used to get more more detailed debug
/// information.
///
/// # Example
///
/// ```rust
/// # use nuuid::Uuid;
/// let uuid = Uuid::parse("662aa7c7-7598-4d56-8bcc-a72c30f998a2").unwrap();
/// assert_eq!(format!("{:?}", uuid), "Uuid(662AA7C7-7598-4D56-8BCC-A72C30F998A2)");
/// assert_eq!(format!("{:#?}", uuid), r#"Uuid(662AA7C7-7598-4D56-8BCC-A72C30F998A2) {
///     Version: Random(4),
///     Variant: Rfc4122(1),
/// }"#);
/// ```
impl fmt::Debug for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(
                f,
                r#"Uuid({:X}) {{
    Version: {}({}),
    Variant: {}({}),
}}"#,
                self,
                self.version(),
                self.version() as u8,
                self.variant(),
                self.variant() as u8
            )
        } else {
            write!(f, "Uuid({:X})", self)
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    const UUID_NIL: &str = "00000000-0000-0000-0000-000000000000";
    const UUID_V4: &str = "662aa7c7-7598-4d56-8bcc-a72c30f998a2";
    const UUID_V4_SIMPLE: &str = "662aa7c775984d568bcca72c30f998a2";
    const UUID_V4_BRACED: &str = "{662aa7c7-7598-4d56-8bcc-a72c30f998a2}";
    const UUID_V4_URN: &str = "urn:uuid:662aa7c7-7598-4d56-8bcc-a72c30f998a2";
    const UUID_V4_URN_UPPER: &str = "urn:uuid:662AA7C7-7598-4D56-8BCC-A72C30F998A2";
    const RAW: [u8; 16] = [
        102, 42, 167, 199, 117, 152, 77, 86, 139, 204, 167, 44, 48, 249, 152, 162,
    ];

    fn name(fun: fn(Uuid, &[u8]) -> Uuid, ver: Version) {
        let namespace = Uuid::new_v4();
        let namespace2 = Uuid::new_v4();
        let uuid1 = fun(namespace, b"test");
        let uuid2 = fun(namespace, b"test");
        assert_eq!(
            uuid1, uuid2,
            "UUID's from different times with the same name/namespace must be equal"
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
    #[cfg(feature = "experimental_uuid")]
    fn new_v6() {
        // Values sourced from https://www.ietf.org/archive/id/draft-peabody-dispatch-new-uuid-format-04.html#name-test-vectors
        const UUID: &str = "1EC9414C-232A-6B00-B3C8-9E6BDECED846";
        let (ticks, counter, node) = (138648505420000000, 13256, [158, 107, 222, 206, 216, 70]);

        let uuid = Uuid::new_v6(ticks, counter, node);
        let uuid_ = Uuid::parse(UUID).unwrap();

        assert_eq!(uuid.to_str_upper(&mut [0; 36]), UUID);
        assert_eq!(uuid.version(), Version::Database);
        assert_eq!(uuid.variant(), Variant::Rfc4122);

        assert_eq!(uuid.timestamp(), uuid_.timestamp());
        assert_eq!(uuid.clock_sequence(), uuid_.clock_sequence());
        assert_eq!(uuid.node()[..], uuid_.node());
    }

    #[test]
    #[cfg(feature = "experimental_uuid")]
    fn new_v7() {
        // Values sourced from https://www.ietf.org/archive/id/draft-peabody-dispatch-new-uuid-format-04.html#name-uuidv7-example-test-vector
        const UUID: &str = "017F22E2-79B0-7CC3-98C4-DC0C0C07398F";
        let (unix_ts, rand_a, rand_b) = (0x17F22E279B0, 0xCC3, 0x18C4DC0C0C07398F);

        let uuid = Uuid::new_v7(unix_ts, rand_a, rand_b);
        let uuid_ = Uuid::parse(UUID).unwrap();

        assert_eq!(uuid.to_str_upper(&mut [0; 36]), UUID);
        assert_eq!(uuid.version(), Version::UnixTime);
        assert_eq!(uuid.variant(), Variant::Rfc4122);

        assert_eq!(uuid.timestamp(), uuid_.timestamp());
        assert_eq!(uuid.clock_sequence(), uuid_.clock_sequence());
        assert_eq!(uuid.node()[..], uuid_.node());
    }

    #[test]
    fn time() {
        use uuid_::{v1::*, Uuid as Uuid_};
        let (ticks, counter, node) = (138788330336896890u64, 8648, *b"world!");

        let uuid = Uuid::new_v1(ticks, counter, node);
        let uuid_ = Uuid_::new_v1(Timestamp::from_rfc4122(ticks, counter), &node);
        assert_eq!(uuid.to_bytes(), *uuid_.as_bytes());
        assert_eq!(uuid.version(), Version::Time);
        assert_eq!(uuid.variant(), Variant::Rfc4122);

        assert_eq!(
            uuid.timestamp(),
            uuid_.get_timestamp().unwrap().to_rfc4122().0
        );
        assert_eq!(
            uuid.clock_sequence(),
            uuid_.get_timestamp().unwrap().to_rfc4122().1
        );
        assert_eq!(uuid.node()[..], uuid_.as_fields().3[2..]);
    }

    #[test]
    fn md5() {
        name(Uuid::new_v3, Version::Md5);
        let uuid = Uuid::new_v3(NAMESPACE_DNS, b"www.widgets.com");
        assert_eq!(
            uuid,
            // From Appendix B, with errata 1352, since RFC is wrong.
            // Because of course it is.
            Uuid::from_str("3d813cbb-47fb-32ba-91df-831e1593ac29").unwrap()
        )
    }

    #[test]
    fn sha1() {
        name(Uuid::new_v5, Version::Sha1)
    }

    #[test]
    fn parse_string() {
        let test = &[UUID_V4, UUID_V4_URN, UUID_V4_BRACED, UUID_V4_SIMPLE];
        for uuid in test {
            println!("Source UUID: {}", uuid);
            let uuid = Uuid::from_str(&uuid.to_ascii_lowercase()).unwrap();
            println!("Parsed UUID: {}\n", uuid);
            assert_eq!(RAW, uuid.to_bytes(), "Parsed UUID bytes don't match");
        }

        for uuid in test {
            println!("Source UUID: {}", uuid);
            let uuid = Uuid::from_str(&uuid.to_ascii_uppercase()).unwrap();
            println!("Parsed UUID: {}\n", uuid);
            assert_eq!(RAW, uuid.to_bytes(), "Parsed UUID bytes don't match");
        }
    }

    #[test]
    fn string() {
        let uuid = Uuid::from_bytes(RAW);
        let mut buf = [0; 45];
        assert_eq!(
            uuid.to_str((&mut buf[..36]).try_into().unwrap()),
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

        assert_eq!(uuid_le, uuid_be);
        assert_ne!(uuid_be.to_bytes_me(), uuid_be.to_bytes());

        // Terrible UUID sourced from my partition table on Linux.
        // Either Linux displays them wrong, or parted stores them wrong.
        // Either way, its swapped.
        const UUID: &str = "20169084-b186-884f-b110-3db2c37eb8b5";
        let uuid = Uuid::parse_me(UUID).unwrap();
        let bad_uuid = Uuid::parse(UUID).unwrap();

        assert_ne!(bad_uuid.version(), Version::Random);
        assert_eq!(uuid.version(), Version::Random);
        assert_eq!(uuid.variant(), Variant::Rfc4122);
        // Cant be equal because endian
        assert_ne!(uuid.to_str(&mut [0; 36]), UUID);
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
