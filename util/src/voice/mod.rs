
#[cfg(feature = "voice-silero")]
pub mod silero_vad;

#[cfg(feature = "voice-webrtc")]
pub mod webrtc_vad;

pub trait VoiceDetector {
    fn is_voice_segment(&mut self, chunk: &[i16]) -> Result<bool, String>;
}