//! Type States for using a [`Uuid`][crate::Uuid] properly according to
//! version and variant.
use core::marker::PhantomData;

mod _priv {
    //! Private internal module
    use super::*;

    /// Seals [`IsRfcUuid`] so only we can implement it
    pub trait RfcSeal {}
    impl RfcSeal for RfcNil {}
    impl RfcSeal for RfcV4 {}
}
use _priv::*;

/// Represents any recognized Version for a
/// [`Variant::Rfc`][crate::Variant::Rfc] UUID
pub trait IsRfcUuid: RfcSeal {}

/// Proper implementations should be done on [`RfcSeal`]
impl<T: RfcSeal> IsRfcUuid for T {}

/// A [`Version::Random`][crate::Version::Random] UUID
#[derive(Default, Clone, Copy)]
pub struct RfcV4 {
    _priv: PhantomData<()>,
}

/// A Nil UUID
#[derive(Default, Clone, Copy)]
pub struct RfcNil {
    _priv: PhantomData<()>,
}

impl RfcV4 {
    pub const fn new() -> Self {
        Self { _priv: PhantomData }
    }
}

impl RfcNil {
    pub const fn new() -> Self {
        Self { _priv: PhantomData }
    }
}
