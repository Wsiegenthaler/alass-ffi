use webrtc_vad::Vad as WebRtcVad;
use webrtc_vad::{SampleRate, VadMode};


const SAMPLE_RATE: SampleRate = SampleRate::Rate8kHz;
const VAD_MODE: VadMode = VadMode::LowBitrate;


pub struct Vad {
    vad: WebRtcVad,
    pub chunk_size: usize
}

impl Vad {
    pub fn new() -> Result<Self, String> {
       let vad = WebRtcVad::new_with_rate_and_mode(SAMPLE_RATE, VAD_MODE);
       let chunk_size = (Self::expected_chunk_millis() as f32 / 1000f32 * Self::expected_sample_rate() as f32) as usize;

       Ok(Vad { vad, chunk_size })
    }
    
    pub fn is_voice_segment(&mut self, chunk: &[i16]) -> Result<bool, String> {
        match self.vad.is_voice_segment(chunk) {
            Ok(is_voice) => Ok(is_voice),
            _ => Err("webrtc-vad unable to process samples!".to_string())
        }
    }

    pub fn expected_sample_rate() -> usize {
        match SAMPLE_RATE {
	    SampleRate::Rate8kHz => 8000,
	    SampleRate::Rate16kHz => 16000,
	    SampleRate::Rate32kHz => 32000,
	    SampleRate::Rate48kHz => 48000
        }
    }

    pub fn expected_chunk_millis() -> usize {
        30
    }
}
