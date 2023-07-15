//! Static UUID Definitions

use crate::{uuid, Uuid};

pub(crate) const UUID_STR_LENGTH: usize = 36;
pub(crate) const UUID_URN_LENGTH: usize = 45;
pub(crate) const UUID_BRACED_LENGTH: usize = 38;
pub(crate) const UUID_SIMPLE_LENGTH: usize = 32;
pub(crate) const UUID_URN: &str = "urn:uuid:";
pub(crate) const UUID_URN_PREFIX: usize = UUID_URN.len();

// TODO: Tests for all namespace UUIDs

/// The predefined DNS namespace, 6ba7b810-9dad-11d1-80b4-00c04fd430c8.
pub const NAMESPACE_DNS: Uuid = uuid!("6ba7b810-9dad-11d1-80b4-00c04fd430c8");

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
