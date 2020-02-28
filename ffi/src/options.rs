use crate::result_codes::*;
use crate::util::*;
use crate::catch_panic;
use crate::SyncOptions;

use std::ptr;

use log::*;

///
/// Creates a new `SyncOptions` instance initialized to default values
/// 
#[catch_panic(ptr::null_mut())]
#[no_mangle]
pub extern "C" fn alass_options_new() -> *mut SyncOptions {
    to_ptr(SyncOptions::default())
}

///
/// Sets the `alass` "interval" parameter
/// 
/// This value represents the smallest recognized unit of time. Smaller numbers make
/// the alignment more accurate, greater numbers make aligning faster. (default `60`)
/// 
#[catch_panic(ALASS_INTERNAL_ERROR)]
#[no_mangle]
pub extern "C" fn alass_options_set_interval(options: *mut SyncOptions, value: i64) -> ResultCode {
    if options.is_null() {
        error!("Invalid parameter: SyncOptions pointer is null");
        return ALASS_INVALID_PARAMS;
    }

    let o = from_ptr(options);
    if value > 0 {
        o.interval = value;
        ALASS_SUCCESS
    } else {
        error!("Invalid parameter: 'interval' must be at least 1 (value={})", value);
        ALASS_INVALID_PARAMS
    }
}

///
/// Sets the `alass` "split_mode" parameter
/// 
/// When true `alass` will attempt alignment assuming the presence of commercial breaks or
/// added/removed scenes. Disabling `split_mode` can make syncing faster but will only correct
/// subtitles whose misalignment is the result of a constant shift. (default `true`)
/// 
#[catch_panic(ALASS_INTERNAL_ERROR)]
#[no_mangle]
pub extern "C" fn alass_options_set_split_mode(options: *mut SyncOptions, value: bool) -> ResultCode {
    if options.is_null() {
        error!("Invalid parameter: SyncOptions pointer is null");
        return ALASS_INVALID_PARAMS;
    }

    let o = from_ptr(options);
    o.split_mode = value;
    ALASS_SUCCESS
}

///
/// The `alass` "split_penalty" parameter
/// 
/// Determines how eager the algorithm is to avoid splitting of the subtitles. A value of 1000
/// means that all lines will be shifted by the same offset, while 0.01 will produce MANY
/// segments with different offsets. Values from 1 to 20 are the most useful subtitles by a
/// constant amount. (default `7.0`)
/// 
#[catch_panic(ALASS_INTERNAL_ERROR)]
#[no_mangle]
pub extern "C" fn alass_options_set_split_penalty(options: *mut SyncOptions, value: f64) -> ResultCode {
    if options.is_null() {
        error!("Invalid parameter: SyncOptions pointer is null");
        return ALASS_INVALID_PARAMS;
    }

    let o = from_ptr(options);
    let value = value;
    if value > 0.0 && value <= 1000.0 {
        o.split_penalty = value;
        ALASS_SUCCESS
    } else {
        error!("Invalid parameter: 'split_penalty' should be in the range (0, 1000]. (value={})", value);
        ALASS_INVALID_PARAMS
    }
}

///
/// Sets the `alass` "speed_optimization" parameter
/// 
/// Greatly speeds up synchronization by sacrificing some accuracy. Set to `null` or zero
/// to disable speed optimization. (default `1.0`)
/// 
#[catch_panic(ALASS_INTERNAL_ERROR)]
#[no_mangle]
pub extern "C" fn alass_options_set_speed_optimization(options: *mut SyncOptions, value: *mut f64) -> ResultCode {
    if options.is_null() {
        error!("Invalid parameter: SyncOptions pointer is null");
        return ALASS_INVALID_PARAMS;
    }

    let o = from_ptr(options);
    let value = from_ptr_safe(value).copied();
    match value {
        Some(v) if v > 0.0 => {
            o.speed_optimization = value;
            ALASS_SUCCESS
        },
        Some(v) if v < 0.0 => {
            error!("Invalid parameter: 'speed_optimization' cannot be negative (value={})", v);
            ALASS_INVALID_PARAMS
        },
        _ => {
            o.speed_optimization = None;
            ALASS_SUCCESS
        }
    }
}

///
/// Whether attempt correction of mismatched framerates
/// 
/// Currently the voice-activity detection isn't accurate enough to support
/// this feature and often results in framerate misdetection. It is recommended
/// to only enable framerate correction when using subtitle generated reference
/// spans. (default `false`)
/// 
#[catch_panic(ALASS_INTERNAL_ERROR)]
#[no_mangle]
pub extern "C" fn alass_options_set_framerate_correction(options: *mut SyncOptions, value: bool) -> ResultCode {
    if options.is_null() {
        error!("Invalid parameter: SyncOptions pointer is null");
        return ALASS_INVALID_PARAMS;
    }

    let o = from_ptr(options);
    o.framerate_correction = value;
    ALASS_SUCCESS
}

///
/// Logs the values of the given `SyncOptions` instance (useful for debugging)
/// 
#[catch_panic]
#[no_mangle]
pub extern "C" fn alass_options_log(options: *mut SyncOptions) {
    if options.is_null() {
        error!("Invalid parameter: SyncOptions pointer is null");
        return
    }

    let o = from_ptr(options);
    let speed_opt = match o.speed_optimization {
        Some(v) => format!("{}", v),
        None => String::from("NO")
    };
    info!("SyncOptions(interval={}, split_mode={}, split_penalty={}, speed_optimization={}, framerate_correction={})",
        o.interval, o.split_mode, o.split_penalty, speed_opt, o.framerate_correction);
}

///
/// Deallocates `SyncOptions` instance
/// 
#[catch_panic]
#[no_mangle]
pub extern "C" fn alass_options_free(options: *mut SyncOptions) {
    if !options.is_null() {
        drop(from_ptr_owned(options))
    }
}