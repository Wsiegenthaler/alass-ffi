use std::ffi::CStr;
use std::os::raw::c_char;

///
/// Dereferences a heap pointer and returns a mutable reference, ensuring 
/// the value will not be dropped when it goes out of scope. Useful for
/// working with heap values recieved from accross an ffi boundary without
/// dropping them when finished. Panics if pointer is `null`.
/// 
pub fn from_ptr<T>(value: *mut T) -> &'static mut T {
    unsafe { Box::leak(Box::from_raw(value)) }
}

///
/// Safe version of `from_ptr` which, if pointer is `null`, will
/// return `None` instead of panicking.
///
pub fn from_ptr_safe<T>(value: *mut T) -> Option<&'static T> {
    if value.is_null() {
        None
    } else {
        Some(from_ptr(value))
    }
}

///
/// Dereferences a heap pointer and returns a boxed instance. Because
/// the returned value is owned, it will be dropped when it goes
/// out of scope. Useful for deallocating heap values received from
/// accross an ffi boundary. Panics if pointer is `null`.
/// 
pub fn from_ptr_owned<T>(value: *mut T) -> Box<T> {
    unsafe { Box::from_raw(value) }
}

///
/// Moves an instance to the heap and returns it's pointer. Value
/// will not be dropped when the return value leaves scope. Useful
/// for leaking values to be sent across an ffi boundary.
/// 
pub fn to_ptr<T>(value: T) -> *mut T {
    Box::into_raw(Box::new(value))
}

/// 
/// Copies the contents of a char buffer into an owned `String`
/// instance. Will return `None` if the pointer is `null`, or if
/// the C string is not a valid UTF-8 sequence.
///
pub fn from_cstring(s: *const c_char) -> Option<String> {
    if s.is_null() {
        None
    } else {
        let cstr = unsafe { CStr::from_ptr(s) };
        cstr.to_str().map(String::from).ok()
    }
}

