//! Error type for nuuid library

/// Error type for [`Uuid`][super::Uuid]
#[derive(Debug)]
pub enum NuuidError {
    /// The provided UUID was not as specified by the RFC
    NotRfc,
}

/// Result type that defaults to [`NuuidError`]
pub type Result<T, E = self::NuuidError> = core::result::Result<T, E>;
