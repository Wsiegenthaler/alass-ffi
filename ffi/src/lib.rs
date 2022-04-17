extern crate alass_util;
extern crate alass_ffi_macros;

mod util;
mod result_codes;
mod options;
mod audio_sink;
mod timespans;
mod voice_activity;
mod logging;

pub use util::*;
pub use result_codes::*;
pub use options::*;
pub use audio_sink::*;
pub use timespans::*;
pub use voice_activity::*;
pub use logging::alass_log_config;

use alass_util::{sync, is_format_supported};
use alass_util::{AudioSink, TimeSpans, SyncOptions, SyncError, SyncError::*};

use alass_ffi_macros::catch_panic;

use log::error;

use std::os::raw::c_char;

///
/// Performs `alass` subtitle synchronization.
/// 
/// * `sub_path_in`: Path to the incorrect subtitle file.
/// 
/// * `sub_path_out`: Path to which the synchronized subtitle file shall
///    be written (must include filename).
/// 
/// * `ref_spans`: Reference timespans to use for alignment.
/// 
/// * `ref_fps`: Framerate of the reference video file (used for framerate correction).
/// 
/// * `sub_encoding`: The IANA charset encoding of the subtitle file. If 'auto' is
///    given (or if not specified), an attempt is made to guess the correct encoding
///    based on the contents of the file.
/// 
/// * `options`: Parameters governing various aspects of the synchronization process. See
///    `SyncOptions` or `alass` documentation for details.
/// 
#[catch_panic(ALASS_INTERNAL_ERROR)]
#[no_mangle]
pub extern "C" fn alass_sync(
        sub_path_in: *const c_char,
        sub_path_out: *const c_char,
        ref_spans: *mut TimeSpans,
        ref_fps: f64,
        sub_encoding: *const c_char,
        options: *mut SyncOptions)
    -> ResultCode {

    if ref_spans.is_null() {
        error!("Invalid parameter: ref_spans is null");
        return ALASS_INVALID_PARAMS;
    }

    let sub_path_in_str = from_cstring(sub_path_in);
    if sub_path_in_str.is_none() {
        error!("Invalid parameter: sub_path_in is invalid");
        return ALASS_INVALID_PARAMS;
    }

    let sub_path_out_str = from_cstring(sub_path_out);
    if sub_path_out_str.is_none() {
        error!("Invalid parameter: sub_path_out is invalid");
        return ALASS_INVALID_PARAMS;
    }

    let sub_encoding_str = from_cstring(sub_encoding);

    let dflt_opts = SyncOptions::default();
    let options = from_ptr_safe(options).unwrap_or(&dflt_opts);

    let ref_spans = from_ptr(ref_spans);
    match sync(&sub_path_in_str.unwrap(), &sub_path_out_str.unwrap(), ref_spans, ref_fps, sub_encoding_str, options)    {
        Ok(_) => ALASS_SUCCESS,
        Err(e) => {
            error!("{}", e);
            sync_error_code(&e)
        }
    }
}

///
/// Determines whether the given subtitle is able to be synced
/// 
/// Support is indicated by either `ALASS_SUCCESS`, `ALASS_UNSUPPORTED_FORMAT`,
/// or a more specific result code if a determination could not be made.
/// 
#[catch_panic(ALASS_INTERNAL_ERROR)]
#[no_mangle]
pub extern "C" fn alass_format_is_supported(sub_path: *const c_char) -> ResultCode {
    let sub_path_str = match from_cstring(sub_path) {
        Some(val) => val,
        None => {
            error!("Invalid parameter: sub_path is invalid");
            return ALASS_INVALID_PARAMS;
        }
    };

    match is_format_supported(&sub_path_str) {
        Ok(()) => ALASS_SUCCESS,
        Err(e) => {
            error!("{}", e);
            sync_error_code(&e)
        }
    }
}

///
/// The sample rate expected by this audio sink (usually either 8kHz or 16kHz).
///
#[catch_panic(-1)]
#[no_mangle]
pub extern "C" fn alass_expected_sample_rate() -> i32 {
    AudioSink::expected_sample_rate() as i32
}

fn sync_error_code(e: &SyncError) -> ResultCode {
    match e {
        UnsupportedFormat { .. } => ALASS_UNSUPPORTED_FORMAT,
        ReadError { .. }         => ALASS_READ_ERROR,
        DoesNotExist { .. }      => ALASS_FILE_DOES_NOT_EXIST,
        PermissionDenied { .. }  => ALASS_PERMISSION_DENIED,
        ParseError { .. }        => ALASS_PARSE_ERROR,
        WriteError { .. }        => ALASS_WRITE_ERROR,
        SerializeError { .. }    => ALASS_SERIALIZE_ERROR,
        InternalError { .. }     => ALASS_INTERNAL_ERROR
    }
}
