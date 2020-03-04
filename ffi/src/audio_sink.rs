extern crate alass_util;

use crate::util::*;
use crate::result_codes::*;
use crate::catch_panic;

use alass_util::{AudioSink, AudioSinkError};

use std::ptr;

use log::error;

///
/// Allocates a new `AudioSink` instance ready to receive audio samples.
/// 
#[catch_panic(ptr::null_mut())]
#[no_mangle]
pub extern "C" fn alass_audio_sink_new() -> *mut AudioSink {
    to_ptr(AudioSink::default())
}

///
/// Send audio samples to `AudioSink` instance. Samples should be 8kHz 16-bit signed
/// little-endian mono.
/// 
#[catch_panic(ALASS_INTERNAL_ERROR)]
#[no_mangle]
pub extern "C" fn alass_audio_sink_send(sink: *mut AudioSink, samples: *mut u8, sample_cnt: i32) -> ResultCode {
    if sink.is_null() {
        error!("Invalid parameter: AudioSink pointer is null");
        return ALASS_INVALID_PARAMS;
    } else if samples.is_null() {
        error!("Invalid parameter: sample buffer pointer is null");
        return ALASS_INVALID_PARAMS;
    } else if sample_cnt < 0 {
        error!("Invalid parameter: sample count is negative");
        return ALASS_INVALID_PARAMS;
    }

    let sink = from_ptr(sink);
    let sample_buf = unsafe { std::slice::from_raw_parts(samples as *mut i16, sample_cnt as usize) };
    match sink.send_samples(sample_buf) {
        Ok(()) => ALASS_SUCCESS,
        Err(e) => {
            error!("{}", e);
            match e {
                AudioSinkError::SinkClosed => ALASS_SINK_CLOSED,
                _ => ALASS_INTERNAL_ERROR
            }
        }
    }
}

///
/// Closes a `AudioSink` instance. Once a sink is closed it can no longer receive additional samples.
/// 
#[catch_panic]
#[no_mangle]
pub extern "C" fn alass_audio_sink_close(sink: *mut AudioSink) {
    if sink.is_null() {
        error!("Invalid parameter: AudioSink pointer is null");
    }

    let _ = from_ptr(sink).close();
}

///
/// Deallocates `AudioSink` instance.
/// 
#[catch_panic]
#[no_mangle]
pub extern "C" fn alass_audio_sink_free(sink: *mut AudioSink) {
    if !sink.is_null() {
        drop(from_ptr_owned(sink));
    }
}
