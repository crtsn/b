use core::ffi::*;
use crate::crust::libc::*;
use crate::ir::{Radix};

#[must_use]
unsafe fn parse_digit(c: u8, radix: Radix) -> u8 {
    if isdigit(c as c_int) != 0 {
        let digit = c as u8 - '0' as u8;

        return digit as u8;
    }
    if matches!(radix, Radix::Hex) {
        let c = tolower(c as c_int) as c_char;
        return c as u8 - 'a' as u8 + 10;
    }
    unreachable!()
}

#[must_use]
pub unsafe fn parse_int_literal_to_u64(value: *const c_char, radix: Radix) -> Option<u64> {
    let count = strlen(value);

    let mut result: u64 = 0;
    for i in 0..count {
      let bytes = CStr::from_ptr(value).to_bytes();
      let digit = parse_digit(bytes[i], radix);
      let Some(r) = result.checked_mul(radix as u64) else {
          return None;
      };
      result = r;

      let Some(r) = result.checked_add(digit as u64) else {
          return None;
      };
      result = r;
    }
    Some(result)
}

#[must_use]
pub unsafe fn parse_char_literal_to_u64_le(char_literal: *const c_char) -> Option<u64> {
  let word_bytes = 8;
  let count = strlen(char_literal);

  let mut result: u64 = 0;
  for i in 0..count {
      if count > word_bytes {
          return None;
      }

      let shift_amount = i * 8;
      let bytes = CStr::from_ptr(char_literal).to_bytes();
      let char_byte = bytes[i];
      let shifted_char = (char_byte as u64) << shift_amount;

      let Some(r) = result.checked_add(shifted_char) else {
          return None;
      };
      result = r;
  }
  Some(result)
}

#[must_use]
pub unsafe fn parse_int_literal_to_u16(value: *const c_char, radix: Radix) -> Result<u16, ()> {
    let count = strlen(value);

    let mut result: u16 = 0;
    for i in 0..count {
      let bytes = CStr::from_ptr(value).to_bytes();
      let digit = parse_digit(bytes[i], radix);
      let Some(r) = result.checked_mul(radix as u16) else {
          return Err(());
      };
      result = r;

      let Some(r) = result.checked_add(digit as u16) else {
          return Err(());
      };
      result = r;
    }
    Ok(result)
}

#[must_use]
pub unsafe fn parse_char_literal_to_u16_be(char_literal: *const c_char) -> Result<u16, ()> {
    let word_bytes = 2;
    let count = strlen(char_literal);

    let mut result: u16 = 0;
    for i in 0..count {
        if count > word_bytes {
            return Err(());
        }

        let shift_amount = (count - 1 - i) * 8;
        let bytes = CStr::from_ptr(char_literal).to_bytes();
        let char_byte = bytes[i];
        let shifted_char = (char_byte as u16) << shift_amount;

        let Some(r) = result.checked_add(shifted_char) else {
            return Err(());
        };
        result = r;
    }
    Ok(result)
}

#[must_use]
pub unsafe fn parse_char_literal_to_u16_le(char_literal: *const c_char) -> Result<u16, ()> {
  let word_bytes = 2;
  let count = strlen(char_literal);

  let mut result: u16 = 0;
  for i in 0..count {
      if count > word_bytes {
          return Err(());
      }

      let shift_amount = i * 8;
      let bytes = CStr::from_ptr(char_literal).to_bytes();
      let char_byte = bytes[i];
      let shifted_char = (char_byte as u16) << shift_amount;

      let Some(r) = result.checked_add(shifted_char) else {
          return Err(());
      };
      result = r;
  }
  Ok(result)
}

#[must_use]
pub unsafe fn parse_int_literal_to_u8(value: *const c_char, radix: Radix) -> Result<u8, ()> {
    let count = strlen(value);

    let mut result: u8 = 0;
    for i in 0..count {
      let bytes = CStr::from_ptr(value).to_bytes();
      let digit = parse_digit(bytes[i], radix);
      let Some(r) = result.checked_mul(radix as u8) else {
          return Err(());
      };
      result = r;

      let Some(r) = result.checked_add(digit as u8) else {
          return Err(());
      };
      result = r;
    }
    Ok(result)
}

#[must_use]
pub unsafe fn parse_char_literal_to_u8(char_literal: *const c_char) -> Result<u8, ()> {
  if strlen(char_literal) > 1 {
      return Err(());
  }

  let bytes = CStr::from_ptr(char_literal).to_bytes();

  if let Some(r) = 0u8.checked_add(bytes[0]) {
    Ok(r)
  } else {
      Err(())
  }
}

