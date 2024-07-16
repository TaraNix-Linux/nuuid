//! Private module that publicly exports all *public* types
//!
//! Exists for code organization purposes

/// Represents a UUID, providing per-version field access and offsets as a
/// union for convenience
///
/// # Considerations
///
/// It is advised by the RFC to treat UUIDs as only opaque blobs of bytes.
/// See S 6.12.
///
/// # Safety
///
/// All variants, and the union as a whole, MUST be:
///
/// - Exactly `16` bytes / 128 bits
/// - Valid for arbitrary bit-patterns at all times
///   - In other words, no uninitialized bytes, ever.
/// - Sound to cast between at all times
#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) union ReprUuid {
    arr: [u8; 16],
    v1: V1,
}

impl ReprUuid {
    /// All-zero
    #[inline]
    pub(crate) const fn nil() -> Self {
        Self { arr: [0u8; 16] }
    }

    /// All-one
    pub(crate) const fn max() -> Self {
        Self { arr: [0xFF; 16] }
    }

    /// Get the variant
    #[inline]
    pub(crate) const fn variant(self) -> Variant {
        // Variant is located in byte 8, highest 4 bits.
        let arr = self.arr();
        let byte = arr[8] >> 4;

        // Done this way for exhaustiveness checking.
        //
        // Doing it this way allows for the compiler to use cmov,
        // making this completely branchless.
        //
        // The simpler range-based match does not do this.
        match (
            // Msb0
            ((byte >> 3) & 1) == 1,
            // Msb1
            ((byte >> 2) & 1) & 1 == 1,
            // Msb2
            ((byte >> 1) & 1) & 1 == 1,
            // // Msb3
            // ((byte >> 0) & 1) & 1 == 1,
        ) {
            // MSB 0, MSB 1, MSB 2, MSB 3
            // MSB 0, x, x, x
            (false, ..) => Variant::Ncs,
            // MSB 0, MSB 1, x, x
            (true, false, ..) => Variant::Rfc,
            // MSB 0, MSB 1, 2, x
            (true, true, false) => Variant::Microsoft,
            // MSB 0, MSB 1, 3, x
            (true, true, true) => Variant::Reserved,
        }
    }

    /// Get the variant. Meaningless if not [`Variant::Rfc`]
    #[inline]
    pub(crate) const fn version(self) -> Version {
        // Variant is located in byte 6, highest 4 bits.
        let arr = self.arr();
        let byte = arr[6] >> 4;

        // Unlike `Uuid::variant`, the simple match works well here,
        // this generates cmov and is branchless.
        //
        // The complicated bool truth table is *very* branch-ful.
        match byte {
            0 => Version::Unused,
            1 => Version::Gregorian,
            2 => Version::Dce,
            3 => Version::Md5,
            4 => Version::Random,
            5 => Version::Sha1,
            6 => Version::Database,
            7 => Version::UnixTime,
            8 => Version::Vendor,
            _ => Version::Reserved,
        }
    }

    #[inline]
    pub(crate) const fn arr(&self) -> &[u8; 16] {
        // Safety: Always
        unsafe { &self.arr }
    }
}

/// UUID Variant. RFC S 4.1.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[non_exhaustive]
pub enum Variant {
    /// Network Computing System (NCS) backward compatibility. Also includes
    /// [`Uuid::nil`][crate::Uuid::nil]
    Ncs,

    /// RFC conforming
    Rfc,

    /// Reserved for Microsoft backward compatibility.
    Microsoft,

    /// Reserved for the future. Also includes [`Uuid::max`][crate::Uuid::max]
    Reserved,
}

/// UUID Version. RFC S 4.2.
// NOTE: !!MUST!! be defined in the same order as bit checks in
// [`ReprUuid::version`] / increasing version number
// This allows the method to optimize to cmov and be branchless,
// because versions have the same repr as this enum
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum Version {
    /// Unused
    Unused,

    /// Version 1, Gregorian Time
    Gregorian,

    /// Version 2, DCE Security
    Dce,

    /// Version 3, MD5 hash
    Md5,

    /// Version 4, Random
    Random,

    /// Version 5, SHA-1 Hash
    Sha1,

    /// Version 6, Database Time / Reordered Gregorian Time
    ///
    /// This is field-compatible with [`Version::Gregorian`],
    /// but reordered for DB locality
    Database,

    /// Version 7, Unix Time
    UnixTime,

    /// Version 8, Application-Specific / Vendor-Specific / Experimental
    ///
    /// Uniqueness is implementation-specific and "MUST NOT be assumed"
    /// RFC S 5.8.
    Vendor,

    /// Reserved versions. Currently, versions 9-15 are reserved
    Reserved,
}

mod _impl {
    //! Private internal module for code organization purposes
    //!
    //! Contains implementations for items in our parent module that
    //! take up a lot of space and are not expected to change much, if ever,
    //! again
    use core::{cmp::Ordering, fmt, hash::Hasher};

    use super::*;

    /// All-zeros
    impl Default for ReprUuid {
        #[inline]
        fn default() -> Self {
            Self::nil()
        }
    }

    impl core::hash::Hash for ReprUuid {
        #[inline]
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.arr().hash(state)
        }
    }

    impl PartialEq for ReprUuid {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.arr().eq(other.arr())
        }
    }

    impl Eq for ReprUuid {}

    impl PartialOrd for ReprUuid {
        #[inline]
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for ReprUuid {
        #[inline]
        fn cmp(&self, other: &Self) -> Ordering {
            self.arr().cmp(other.arr())
        }
    }

    impl fmt::Display for Version {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{self:?}")
        }
    }

    impl fmt::Display for Variant {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{self:?}")
        }
    }
}
