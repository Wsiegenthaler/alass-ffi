extern crate alass_util;

use crate::util::*;
use crate::result_codes::*;
use crate::catch_panic;

use std::ptr;
use std::os::raw::c_char;

use alass_util::*;
use alass_util::TimeSpansSaveError::*;

use subparse::timetypes::TimeSpan as SubTimeSpan;
use subparse::timetypes::TimePoint as SubTimePoint;

use log::error;

static DEFAULT_SPANS_CAPACITY: usize = 2000;

///
/// Creates a new timespans buffer ready to accept data (see `alass_timespans_push()`).
/// 
#[catch_panic(ptr::null_mut())]
#[no_mangle]
pub extern "C" fn alass_timespans_new() -> *mut TimeSpans {
    to_ptr(TimeSpans(Vec::with_capacity(DEFAULT_SPANS_CAPACITY)))
}

///
/// Appends timespan to buffer. Start and end times are in milliseconds.
/// 
#[catch_panic(ALASS_INTERNAL_ERROR)]
#[no_mangle]
pub extern "C" fn alass_timespans_push(spans: *mut TimeSpans, start_time: i64, end_time: i64) -> ResultCode {
    if spans.is_null() {
        error!("Invalid parameter: spans is null");
        return ALASS_INVALID_PARAMS;
    }

    if start_time > end_time {
        error!("Invalid parameter: invalid timespan");
        return ALASS_INVALID_PARAMS;
    }

    let spans = from_ptr(spans);
    let start = SubTimePoint::from_msecs(start_time);
    let end = SubTimePoint::from_msecs(end_time);
    spans.push(SubTimeSpan::new(start, end));

    ALASS_SUCCESS
}

///
/// Computes timespans given detected voice-activity.
/// 
#[catch_panic(ptr::null_mut())]
#[no_mangle]
pub extern "C" fn alass_timespans_compute(activity: *mut VoiceActivity) -> *mut TimeSpans {
    if activity.is_null() {
        error!("Invalid parameter: voice activity pointer is null");
        return ptr::null_mut()
    }

    let activity = &*from_ptr(activity);
    let spans = TimeSpans::from(activity);
    to_ptr(spans)
}

///
/// Saves timespans to disk with the given `filename` (see `alass_timespans_load_raw()`).
/// 
#[catch_panic(ALASS_INTERNAL_ERROR)]
#[no_mangle]
pub extern "C" fn alass_timespans_save_raw(spans: *mut TimeSpans, filename: *const c_char) -> ResultCode {
    if spans.is_null() {
        error!("Invalid parameter: spans is null");
        return ALASS_INVALID_PARAMS;
    }

    let filename_str = from_cstring(filename);
    if filename_str.is_none() {
        error!("Invalid parameter: filename is invalid");
        return ALASS_INVALID_PARAMS;
    }

    let filename_str = filename_str.unwrap();
    let spans = from_ptr(spans);
    match spans.save(&filename_str) {
        Ok(()) => ALASS_SUCCESS,
        Err(e) => {
            error!("{}", e);
            match e {
                SerializeError { .. } => ALASS_SERIALIZE_ERROR,
                WriteError { .. } => ALASS_WRITE_ERROR
            }
        }
    }
}

///
/// Loads cached timespans from disk (see `alass_timespans_save_raw()`). Returns
/// null if no file exists at the given path.
/// 
#[catch_panic(ptr::null_mut())]
#[no_mangle]
pub extern "C" fn alass_timespans_load_raw(filename: *const c_char) -> *mut TimeSpans {
    let filename_str = from_cstring(filename);
    if filename_str.is_none() {
        error!("Invalid parameter: filename is invalid");
        return ptr::null_mut();
    }

    match TimeSpans::load(&filename_str.unwrap()) {
        Ok(spans) => to_ptr(spans),
        Err(e) => {
            error!("{}", e);
            ptr::null_mut()
        }
    }
}

///
/// Loads timespans from subtitle file. Returns null if no file exists at the given path.
/// 
#[catch_panic(ptr::null_mut())]
#[no_mangle]
pub extern "C" fn alass_timespans_load_subtitle(filename: *const c_char, sub_encoding: *const c_char) -> *mut TimeSpans {
    let filename_str = from_cstring(filename);
    if filename_str.is_none() {
        error!("Invalid parameter: filename is invalid");
        return ptr::null_mut();
    }

    let sub_encoding_str = from_cstring(sub_encoding);

    match open_sub_file(&filename_str.unwrap(), sub_encoding_str) {
        Ok((sub_file, _)) => match TimeSpans::from_sub_file(&sub_file) {
            Ok(spans) => to_ptr(spans),
            Err(e) => {
                error!("{}", e);
                ptr::null_mut()
            }
        },
        Err(e) => {
            error!("{}", e);
            ptr::null_mut()
        }
    }
}

///
/// Deallocates timespans buffer.
/// 
#[catch_panic]
#[no_mangle]
pub extern "C" fn alass_timespans_free(spans: *mut TimeSpans) {
    if !spans.is_null() {
        drop(from_ptr_owned(spans));
    }
}
