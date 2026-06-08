use core::ffi::*;
use crate::crust::libc::*;

pub const MAX_ERROR_COUNT: usize = 100;

/// The point of this function is to indicate that a compilation error happened, but continue the compilation anyway
/// even if the state of the Compiler became bogus. This is needed to report as many compilation errors as possible.
/// After calling this function always continue the compilation like nothing happened.
#[must_use]
pub unsafe fn bump_error_count(error_count: *mut usize) -> Option<()> {
    (*error_count) += 1;
    if (*error_count) >= MAX_ERROR_COUNT {
        fprintf(stderr(), c!("TOO MANY ERRORS! Fix your program!\n"));
        return None
    }
    Some(())
}


