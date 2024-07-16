//! Create and use UUIDs
#![no_std]
#![deny(
    //
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::private_doc_tests,
    // missing_docs,
)]
// Project Lints
#![allow(clippy::identity_op)]
// Development Lints
// #![allow(
//     //
//     unused_imports,
//     dead_code,
//     unreachable_code,
//     unused_variables
// )]
use core::{fmt, marker::PhantomData};

use crate::state::IsRfcUuid;

pub mod error;
pub mod state;

mod types;
pub use crate::types::*;

/// Universally Unique Identifier, or UUID.
///
/// # Considerations
///
/// It is advised by the RFC to treat UUIDs as only opaque blobs of bytes.
/// See S 6.12. Using fields is error-prone.
///
/// # Safety
///
/// This type is `repr(transparent)` and guaranteed to have the same layout
/// as `[u8; 16]`.
/// It is sound for code to treat this type as a byte array of the same size,
/// or treat one as a reference to us.
///
/// RFC "fields" are variant and version dependent. They are assumed to be laid
/// out Most Significant Byte First/MSB/Big-Endian/Network Endian.
#[repr(transparent)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
pub struct Uuid<State = state::RfcNil> {
    uuid: ReprUuid,
    state: PhantomData<State>,
}

/// Methods available on any variant / version
// Public API - Information - Any Variant / Version
impl<S> Uuid<S> {
    /// Get the variant
    #[inline]
    pub const fn variant(&self) -> Variant {
        self.uuid.variant()
    }

    /// Represent this UUID as an opaque byte array
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        self.uuid.arr()
    }
}

/// Methods available on RFC UUIDs of any version
// Public API - Creation - Any Version
impl<S: IsRfcUuid> Uuid<S> {
    /// The special all-zero / "nil" UUID. S 5.9.
    #[inline]
    pub const fn nil() -> Self {
        Self {
            uuid: ReprUuid::nil(),
            state: PhantomData,
        }
    }

    /// The special all-one / "max" UUID. S 5.10.
    #[inline]
    pub const fn max() -> Self {
        Self {
            uuid: ReprUuid::max(),
            state: PhantomData,
        }
    }

    /// Get the [`Version`]
    #[inline]
    pub const fn version(&self) -> Version {
        self.uuid.version()
    }
}

// #[cfg(no)]
#[allow(unused_variables, unreachable_code)]
impl<S> fmt::Debug for Uuid<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
        // TODO: debug formatting
        write!(f, "Uuid(")?;
        match self.variant() {
            Variant::Ncs => write!(f, "Ncs")?,
            Variant::Rfc => {
                //
                write!(f, "Rfc(")?;
                // match self.variant() {}
                write!(f, ")")?;
            }
            Variant::Microsoft => write!(f, "Microsoft")?,
            Variant::Reserved => write!(f, "Reserved")?,
        }
        write!(f, ":")?;
        // TODO: String UUID
        write!(f, ")")?;
        Ok(())
    }
}

// #[cfg(no)]
impl<S> fmt::Display for Uuid<S> {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: display formatting
        todo!();
        // Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::mem::size_of;

    use state::RfcV4;

    use super::*;

    /// Static compile-time tests
    const _: () = {
        assert!(
            size_of::<ReprUuid>() == 16,
            "`RawUuid` must be exactly 16 bytes / 128 bits"
        );
        assert!(
            size_of::<Uuid<ReprUuid>>() == 16,
            "`Uuid<S>` must be exactly 16 bytes / 128 bits"
        );
    };
}
