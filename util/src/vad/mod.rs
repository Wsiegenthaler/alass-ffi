
#[cfg(feature = "vad-silero-onnxruntime")]
pub mod silero_onnxruntime;

#[cfg(feature = "vad-silero-tract")]
pub mod silero_tract;

#[cfg(feature = "vad-webrtc")]
pub mod webrtc;

// Only one voice detector feature can be enabled at a time
#[cfg(any(
    all(feature = "vad-webrtc", feature = "vad-silero-onnxruntime"),
    all(feature = "vad-webrtc", feature = "vad-silero-tract"),
    all(feature = "vad-silero-tract", feature = "vad-silero-onnxruntime")
))]
compile_error!("Only one Voice Auto Detector (VAD) can be enabled at a time (either Cargo feature \"vad-webrtc\", \"vad-silero-tract\", OR \"vad-silero-onnxruntime\")");
