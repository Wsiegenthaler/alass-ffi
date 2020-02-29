extern crate alass_util;

use crate::util::*;
use crate::catch_panic;

use alass_util::{AudioSink, VoiceActivity};

use std::ptr;

use log::error;

///
/// Computes voice activity given an `AudioSink` containing sample data.
/// 
#[catch_panic(ptr::null_mut())]
#[no_mangle]
pub extern "C" fn alass_voice_activity_compute(sink: *mut AudioSink) -> *mut VoiceActivity {
    if sink.is_null() {
        error!("Invalid parameter: AudioSink pointer is null");
        return ptr::null_mut()
    }

    let sink = from_ptr(sink);
    let activity = sink.voice_activity();
    to_ptr(activity)
}

///
/// [EXPERIMENTAL] Cleans voice-activity data
/// 
/// This operation successively employs mathematical morphological 'erosion'
/// and 'dilation` operators to clean the output of the voice-activity detector.
/// The result is a clone of the original `VoiceActivity` instance having
/// cleaner/fewer timespans.
/// 
/// The `opening_radius` and `closing_radius` parameters represent the kernel radii
/// of the mathematical morphological operators. Each radius determines a window
/// of size `(2r+1)*CHUNK_MILLIS` milliseconds. Any errant spans smaller than this
/// window will be removed and any gaps larger than this window will be filled.
/// 
#[catch_panic(ptr::null_mut())]
#[no_mangle]
pub extern "C" fn alass_voice_activity_clean(activity: *mut VoiceActivity, opening_radius: usize, closing_radius: usize) -> *mut VoiceActivity {
    if activity.is_null() {
        error!("Invalid parameter: voice activity pointer is null");
        return ptr::null_mut()
    }

    let activity = &*from_ptr(activity);
    to_ptr(activity.clean(opening_radius, closing_radius))
}

///
/// Deallocates `VoiceActivity` instance.
/// 
#[catch_panic]
#[no_mangle]
pub extern "C" fn alass_voice_activity_free(activity: *mut VoiceActivity) {
    if !activity.is_null() {
        drop(from_ptr_owned(activity));
    }
}
