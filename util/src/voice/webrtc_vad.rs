use webrtc_vad::{Vad, SampleRate, VadMode};

use crate::voice::VoiceDetector;


struct WebRtcVad {
    vad: Vad
}

impl WebRtcVad {
    fn new(rate: SampleRate, mode: VadMode) -> Self {
       WebRtcVad {
           vad: Vad::new_with_rate_and_mode(rate, mode)
       }
    }
}

impl VoiceDetector for WebRtcVad {
    fn is_voice_segment(self, chunk: &[i16]) -> Option<bool> {
        match self.vad.is_voice_segment(chunk) {
            Ok(is_voice) => Some(is_voice),
            Err(_) => None
        }
    }
}