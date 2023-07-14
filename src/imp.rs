//! Private Implementation Details

use core::{
    hint::unreachable_unchecked,
    ops::{Range, RangeFrom},
    slice::from_raw_parts,
};

use crate::{ParseUuidError, UUID_SIMPLE_LENGTH, UUID_STR_LENGTH};

/// Const version of RangeFrom
pub const fn const_range_from(bytes: &[u8], range: RangeFrom<usize>) -> &[u8] {
    const_range(
        bytes,
        Range {
            start: range.start,
            end: bytes.len(),
        },
    )
}

/// Const version of Range
pub const fn const_range(bytes: &[u8], range: Range<usize>) -> &[u8] {
    let len = bytes.len();
    let start = range.start;
    let end = range.end;

    if (start > end) || (end > len) {
        // Trigger a standard indexing panic
        let _ = bytes[start];
        let _ = bytes[end];
        // if that didn't panic it was in-bounds but wrong
        panic!("Invalid const Range")
    }

    // Safety:
    // - We check the range is in bounds and then create a sub-slice from it, stable
    //   and const
    unsafe { from_raw_parts(bytes.as_ptr().add(start), end - start) }
}

const fn _const_is_ascii_hex_dash(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() {
        if !matches!(bytes[i], b'0'..=b'9' | b'A'..=b'F' | b'a'..=b'f' | b'-') {
            return false;
        }
        i += 1;
    }
    true
}

/// Stable const version of slice get_unchecked
pub const unsafe fn const_get_unchecked(bytes: &[u8], idx: usize) -> u8 {
    // Safety: Internal function, statically known to be used only with valid values
    unsafe { *bytes.as_ptr().add(idx) }
}

const fn decode_digit(b: u8) -> Result<u8, ParseUuidError> {
    Ok(match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        b'-' => u8::MAX,
        _ => {
            return Err(ParseUuidError::new());
        }
    })
}

/// Decode a hex string in stable const Rust
///
/// This is very slow compared to what can be done at runtime.
pub const fn const_hex_decode(bytes: &[u8]) -> Result<[u8; 16], ParseUuidError> {
    let len = bytes.len();

    // `bytes` length cannot be anything except these two lengths
    if !(len == UUID_SIMPLE_LENGTH || len == UUID_STR_LENGTH) {
        // panic!("Should be impossible");
        // Safety: This is an internal function and this condition is statically known
        // to be impossible
        unsafe { unreachable_unchecked() }
    }

    let mut out = [0u8; 16];
    let mut i_out = 0;

    let mut i = 0;
    while i < len {
        let b = bytes[i];
        // Safety: len is divisible by 2 and increments by 2, there will always be a
        // next element.
        let b2 = unsafe { const_get_unchecked(bytes, i + 1) };

        #[cfg(no)]
        let h = match decode_digit(b) {
            Ok(u8::MAX) => {
                i += 1;
                continue;
            }
            Ok(b) => b,
            Err(e) => return Err(e),
        };

        #[cfg(no)]
        let l = match decode_digit(b2) {
            Ok(b) => b,
            Err(e) => return Err(e),
        };

        // #[cfg(no)]
        let h = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            b'-' => {
                i += 1;
                continue;
            }
            _ => {
                return Err(ParseUuidError::new());
            }
        };

        // #[cfg(no)]
        let l = match b2 {
            b'0'..=b'9' => b2 - b'0',
            b'a'..=b'f' => b2 - b'a' + 10,
            b'A'..=b'F' => b2 - b'A' + 10,
            _ => {
                return Err(ParseUuidError::new());
            }
        };

        i += 2;
        if i_out < 16 {
            out[i_out] = (h << 4) | l;
            i_out += 1;
        }
    }

    Ok(out)
}
