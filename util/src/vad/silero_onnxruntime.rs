use std::vec;
use std::include_bytes;

use onnxruntime::{GraphOptimizationLevel, OrtError};
use onnxruntime::environment::Environment;
use onnxruntime::ndarray::*;
use onnxruntime::session::Session;
use onnxruntime::tensor::OrtOwnedTensor;
use onnxruntime::LoggingLevel;


const PROB_THRESHOLD: f32 = 0.7;

pub struct Vad {
    model: Session,
    h: ArrayD<f32>,
    c: ArrayD<f32>,
    pub chunk_size: usize
}

impl Vad {

    pub fn new<'a>() -> Result<Self, String> {
        const ERROR_MSG: &str = "Unable to initialize voice auto detector!";

        // Initial model state
        let h = ArrayD::<f32>::zeros(IxDyn(&[2, 1, 64]));
        let c = ArrayD::<f32>::zeros(IxDyn(&[2, 1, 64]));

        let chunk_size = (Self::expected_chunk_millis() as f32 / 1000f32 * Self::expected_sample_rate() as f32) as usize;

        // Model
        let model = Environment::builder()
            .with_name("alass-util")
            .with_log_level(LoggingLevel::Verbose)
            .build().map_err(|_| ERROR_MSG)?
            .new_session_builder().map_err(|_| ERROR_MSG)?
            .with_optimization_level(GraphOptimizationLevel::All).map_err(|_| ERROR_MSG)?
            .with_number_threads(4).map_err(|_| ERROR_MSG)?
            .with_model_from_memory(include_bytes!("silero_net.onnx")).map_err(|_| ERROR_MSG)?;

        Ok(Vad { model, h, c, chunk_size })
    }

    fn process_chunk(&mut self, chunk: &[i16]) -> Result<bool, OrtError> {
        let input = Array2::<f32>::from_shape_fn((1, self.chunk_size), |(_, d)| chunk[d] as f32);
        let args = vec![input.into_dyn(), self.h.clone().into_dyn(), self.c.clone().into_dyn()];
        let result: Vec<OrtOwnedTensor<f32, IxDyn>> = self.model.run(args)?;
        
        // Extract results/state
        let probability = result[0][[0, 1, 0]];
        self.h = result[1].to_owned().into_dyn();
        self.c = result[2].to_owned().into_dyn();

        // Determine voice activity
        Ok(PROB_THRESHOLD <= probability)
    }

    pub fn is_voice_segment(&mut self, chunk: &[i16]) -> Result<bool, String> {
        match self.process_chunk(chunk) {
            Ok(is_voice) => Ok(is_voice),
            Err(err) => Err(format!("The silero-vad module encountered an error while processing samples for voice activity. ({})", err))
        }
    }

    pub fn expected_sample_rate() -> usize {
        16000
    }

    pub fn expected_chunk_millis() -> usize {
        96
    }
}
