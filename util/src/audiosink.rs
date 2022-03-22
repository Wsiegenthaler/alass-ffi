
use std::vec;
use std::fs::File;
use std::io;
use std::io::{BufReader, prelude::*};
use std::{error::Error, fmt};

#[cfg(any(feature = "debug-sample-data", feature = "debug-voice-activity-data"))]
use std::{io::Write, path::PathBuf};

#[cfg(feature = "debug-sample-data")]
use std::slice;

use log::error;
use webrtc_vad::{Vad, SampleRate, VadMode};
use byteorder::{ByteOrder, LittleEndian};

use crate::VoiceActivity;

use AudioSinkError::*;

/// The sensitivity of the WebRTC voice-activity detector. Lowering the sensitivity
/// greatly reduces false positives but increases the chance of missed detections.
const VAD_MODE: VadMode = VadMode::LowBitrate;

/// The sample rate expected by the voice-activity detector
const VAD_SAMPLE_RATE: SampleRate = SampleRate::Rate8kHz;

/// The millisecond duration of each chunk to be processed by the voice-activity
/// detector. The WebRTC VAD expects chunks of 10, 20, or 30ms.
const CHUNK_MILLIS: usize = 30;

/// The number of samples per chunk to be processed by the voice-activity detector
const CHUNK_SAMPLES: usize = CHUNK_MILLIS * VAD_SAMPLE_RATE as usize / 1000;

///
/// Receives audio samples to be processed for voice-activity and used as reference
/// by the synchronization process
/// 
pub struct AudioSink {
    pub state: AudioSinkState,
    sample_buffer: Vec<i16>,
    vad: Vad,
    vad_buffer: Vec<bool>,

    #[cfg(feature = "debug-sample-data")]
    sample_file: File,

    #[cfg(feature = "debug-voice-activity-data")]
    vad_file: File,
}

impl Default for AudioSink {

    ///
    /// Creates a new `AudioSink` instance ready to accept sample data
    /// 
    fn default() -> Self {
        AudioSink {
            state: AudioSinkState::Open,
            sample_buffer: Vec::new(),
            vad: Vad::new_with_rate_and_mode(VAD_SAMPLE_RATE, VAD_MODE),
            vad_buffer: Vec::new(),

            #[cfg(feature = "debug-sample-data")]
            sample_file: AudioSink::create_debug_file("alass-sample-data.raw"),

            #[cfg(feature = "debug-voice-activity-data")]
            vad_file: AudioSink::create_debug_file("alass-voice-activity-data")
        }

    }
}

impl AudioSink {

    ///
    /// Recieve incoming samples
    /// 
    /// Voice-activity data is processed on the fly in `CHUNK_SAMPLES` sized chunks. Remaining
    /// samples are buffered until the next invocation or the `AudioSink` is closed.
    /// 
    pub fn send_samples(self: &mut AudioSink, samples: &[i16]) -> Result<(), AudioSinkError> {
        if self.state == AudioSinkState::Open {
            if self.sample_buffer.len() + samples.len() >= CHUNK_SAMPLES {

                // Combine sink and incoming samples to produce first complete chunk (copy)
                let mut first_chunk = Vec::with_capacity(CHUNK_SAMPLES);
                first_chunk.extend_from_slice(self.sample_buffer.as_slice());
                let len2 = CHUNK_SAMPLES - self.sample_buffer.len();
                first_chunk.extend_from_slice(&samples[0..len2]);
                self.process_chunk(&first_chunk)?;

                // Split the rest of the incoming samples into exactly sized chunks (no copy)
                let remaining_chunks = samples[len2..].chunks_exact(CHUNK_SAMPLES);

                // Save the remainder of the incoming samples to sample_buffer for next call (copy)
                self.sample_buffer.clear();
                self.sample_buffer.extend_from_slice(remaining_chunks.remainder());

                // Process exactly sized chunks (no copy)
                for chunk in remaining_chunks {
                    self.process_chunk(chunk)?;
                }

            } else {
                // Not enough data for a complete chunk, append samples to sample_buffer for next call
                self.sample_buffer.extend_from_slice(samples);
            }
            Ok(())
        } else {
            Err(AudioSinkError::SinkClosed)
        }
    }

    ///
    /// Processes a single chunk of samples for voice activity
    /// 
    /// Chunk must be exactly `CHUNK_SAMPLES` in length.
    /// 
    fn process_chunk(self: &mut AudioSink, chunk: &[i16]) -> Result<(), AudioSinkError> {
        if chunk.len() != CHUNK_SAMPLES {
            error!("Error processing samples: chunk length must be exactly CHUNK_SAMPLES ({})", CHUNK_SAMPLES);
            return Err(VoiceDetectionError)
        }

        // Detect voice activity
        let vad_result = self.vad.is_voice_segment(chunk);
        match vad_result {
            Ok(is_voice) => {
                // Store voice activity for this chunk to buffer
                self.vad_buffer.push(is_voice);

                // Dump voice activity data to file for debugging
                #[cfg(feature = "debug-voice-activity-data")]
                AudioSink::dump_vad(&is_voice, &mut self.vad_file);
            },
            Err(_) => return Err(VoiceDetectionError)
        }

        // Dump samples to file for debugging
        #[cfg(feature = "debug-sample-data")]
        AudioSink::dump_samples(chunk, &mut self.sample_file);

        Ok(())
    }

    ///
    /// Closes the `AudioSink`
    /// 
    /// This flushes any remaining samples and finishes processing voice-activity.
    /// `AudioSink` will no longer accept samples once closed.
    /// 
    pub fn close(self: &mut AudioSink) -> Result<(), AudioSinkError> {
        if self.state == AudioSinkState::Open {
            let buf_len = self.sample_buffer.len();
            if buf_len > 0 {
                let chunk = &mut vec![0i16; CHUNK_SAMPLES];
                chunk[..buf_len].clone_from_slice(self.sample_buffer.as_slice());
                self.process_chunk(chunk.as_slice())?;
            }
            self.state = AudioSinkState::Closed
        }
        Ok(())
    }

    ///
    /// Returns voice-activity data, closing the `AudioSink` if it has not been already
    /// 
    pub fn voice_activity(self: &mut Self) -> VoiceActivity {
        let _ = self.close();
        VoiceActivity { data: self.vad_buffer.clone(), chunk_millis: CHUNK_MILLIS as u64 }
    }

    #[cfg(any(feature = "debug-sample-data", feature = "debug-voice-activity-data"))]
    fn create_debug_file(filename: &str) -> File {
        let mut path = match std::env::var_os("LIBALASS_DEBUG_DATA_DIR") {
            Some(dir) => PathBuf::from(dir),
            None => match std::env::current_dir() {
                Ok(dir) => dir,
                Err(e) => panic!("[alass-util] Unable to obtain current working directory in order to dump raw sample data! (msg='{}')", e)
            }
        };
        path.push(String::from(filename));
        File::create(path).unwrap()
    }

    #[cfg(feature = "debug-sample-data")]
    fn dump_samples(chunk: &[i16], file: &mut File) {
        let bytes: &[u8] = unsafe { slice::from_raw_parts(chunk.as_ptr() as *const u8, chunk.len()*2) };
        match file.write_all(bytes) {
            Ok(_) => (),
            Err(e) => panic!("[libalass] Error writing raw sample data to file! (msg='{}')", e)
        }
    }

    #[cfg(feature = "debug-voice-activity-data")]
    fn dump_vad(is_voice: &bool, file: &mut File) {
        let bytes: &[u8] = &[ if *is_voice { 1_u8 } else { 0u8 } ];
        match file.write_all(bytes) {
            Ok(_) => (),
            Err(e) => panic!("[libalass] Error writing voice activity data to file! (msg='{}')", e)
        }
    }

    ///
    /// Loads sample data from file (for debugging)
    /// 
    pub fn load_sample_data(filename: &str) -> Result<Vec<i16>, io::Error> {
        let file = File::open(String::from(filename))?;
        let bytes = &mut vec![];
        BufReader::new(file).read_to_end(bytes)?;
        let samples = bytes.as_slice().chunks_exact(2).map(LittleEndian::read_i16).collect();
        Ok(samples)
    }

    ///
    /// Loads voice-activity data from file (for debugging)
    /// 
    pub fn load_vad_data(filename: &str) -> Result<Vec<bool>, io::Error> {
        let file = File::open(String::from(filename))?;
        let bytes = &mut vec![];
        BufReader::new(file).read_to_end(bytes)?;
        let vad_data = bytes.iter().map(|b| *b != 0).collect();
        Ok(vad_data)
    }
}

///
/// Represents current state of audio `AudioSink`
/// 
#[derive(PartialEq)]
pub enum AudioSinkState { Open, Closed }

///
/// Represents an error processing sample data
/// 
#[derive(Debug)]
pub enum AudioSinkError {
    SinkClosed,
    SinkOpen,
    VoiceDetectionError
}

impl Error for AudioSinkError {}

impl fmt::Display for AudioSinkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AudioSinkError::SinkClosed => write!(f, "Cannot write samples to sink after it's been closed"),
            AudioSinkError::SinkOpen => write!(f, "Cannot cannot access voice-activity data until sink has been closed"),
            AudioSinkError::VoiceDetectionError => write!(f, "An error occurred during voice-detection")
        }
    }
}
